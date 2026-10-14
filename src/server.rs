//! # NAPI bindings that bridge Rocket's runtime to TypeScript handlers.
//!
//! Rocket's real codegen (`#[get]`, `routes!`, `derive(..)`, `uri!`) are Rust
//! *procedural macros* and cannot cross the N-API boundary. This module is the
//! NAPI equivalent of the *responsibilities* those macros perform:
//!
//! * `#[get("/p")]` / `routes!`  -> [`serve`] + the route-table parser, which at
//!   runtime build the same `&[Route]`-equivalent that Rocket's codegen emits,
//!   except the handler bodies live in JavaScript.
//! * `uri!` type-checked construction -> the TS-side `uri()` builder (see the
//!   codegen layer); this module only carries the request/response wire types.

use std::collections::HashMap;
use std::io::Cursor;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Arc, Mutex, OnceLock};

use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
use napi::bindgen_prelude::Object as JsObject;
#[allow(deprecated)]
use napi::{JsFunction, Unknown};
use napi_derive::napi;

use rocket::config::Config;
use rocket::data::ByteUnit;
use rocket::fairing::AdHoc;
use rocket::http::{Header as RocketHeader, Method, Status as RocketStatus};
use rocket::request::Request;
use rocket::response::Response as RocketResponse;
use rocket::route::{Handler, Outcome as RouteOutcome, Route};
use rocket::{Data, Shutdown};

/// Convenience alias for the threadsafe function each JS route handler is wrapped in.
type HandlerTsfn = ThreadsafeFunction<
    RequestInput,
    Either<Promise<ResponsePayload>, ResponsePayload>,
    (serde_json::Value,),
    Status,
    true,
    false,
    0,
>;

#[allow(deprecated)]
fn to_js_function(handler: Unknown) -> JsFunction {
    debug_assert_eq!(
        std::mem::size_of::<Unknown>(),
        std::mem::size_of::<JsFunction>()
    );
    unsafe { std::mem::transmute_copy::<Unknown, JsFunction>(&handler) }
}

/// Internal request structure passed across Tokio threads to the JS thread.
pub struct RequestInput {
    pub method: String,
    pub path: String,
    pub query: String,
    pub headers: Vec<Header>,
    pub body: Option<Buffer>,
    pub params: HashMap<String, String>,
}

/// A single HTTP header.
#[napi(object)]
#[derive(Clone, Debug)]
pub struct Header {
    pub name: String,
    pub value: String,
}

/// Init passed to the `Response` constructor.
#[napi(object)]
pub struct ResponseInit {
    pub status: Option<u32>,
    pub headers: Option<Vec<Header>>,
}

/// Structural response payload returned from JS to Rust.
#[napi(object)]
pub struct ResponsePayload {
    pub status: u32,
    pub headers: Vec<Header>,
    pub body: Option<Buffer>,
}

/// The `Response` class constructed in JS.
#[napi]
pub struct Response {
    pub status: u32,
    headers: Vec<Header>,
    body: Option<Vec<u8>>,
}

#[napi]
impl Response {
    #[napi(getter)]
    pub fn headers(&self) -> Vec<Header> {
        self.headers.clone()
    }

    #[napi(setter)]
    pub fn set_headers(&mut self, headers: Vec<Header>) {
        self.headers = headers;
    }

    #[napi(getter)]
    pub fn body(&self) -> Option<Buffer> {
        self.body.as_ref().map(|b| Buffer::from(b.clone()))
    }

    #[napi(setter)]
    pub fn set_body(&mut self, body: Option<Buffer>) {
        self.body = body.map(|b| b.to_vec());
    }

    #[napi(constructor)]
    pub fn new(body: Option<String>, init: Option<ResponseInit>) -> Self {
        let init = init.unwrap_or(ResponseInit {
            status: None,
            headers: None,
        });
        let body = body.map(|s| s.into_bytes());
        Response {
            status: init.status.unwrap_or(200),
            headers: init.headers.unwrap_or_default(),
            body,
        }
    }

    #[napi]
    pub fn json(data: serde_json::Value, init: Option<ResponseInit>) -> napi::Result<Self> {
        let init = init.unwrap_or(ResponseInit {
            status: None,
            headers: None,
        });
        let bytes = serde_json::to_vec(&data)
            .map_err(|e| napi::Error::new(Status::GenericFailure, e.to_string()))?;
        let mut headers = init.headers.unwrap_or_default();
        headers.push(Header {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        });
        Ok(Response {
            status: init.status.unwrap_or(200),
            headers,
            body: Some(bytes),
        })
    }

    #[napi]
    pub fn redirect(url: String, status: Option<u32>) -> Self {
        Response {
            status: status.unwrap_or(302),
            headers: vec![Header {
                name: "Location".to_string(),
                value: url,
            }],
            body: None,
        }
    }
}

/// Static file marker.
#[napi(object)]
pub struct StaticFile {
    pub path: String,
}

#[napi]
pub fn file(path: String) -> StaticFile {
    StaticFile { path }
}

#[derive(Clone)]
enum RouteKind {
    Fn(Arc<HandlerTsfn>),
    Static(Arc<ResponsePayload>),
    File {
        path: String,
        _cache: Arc<Mutex<Option<Vec<u8>>>>,
    },
}

#[derive(Clone)]
struct JsHandler {
    kind: Arc<RouteKind>,
    pattern: String,
}

#[rocket::async_trait]
impl Handler for JsHandler {
    async fn handle<'r>(&self, request: &'r Request<'_>, data: Data<'r>) -> RouteOutcome<'r> {
        match &*self.kind {
            RouteKind::Fn(tsfn) => {
                let input = build_request(request, data, &self.pattern).await;
                let output = match tsfn.call_async(Ok(input)).await {
                    Ok(Either::A(promise)) => match promise.await {
                        Ok(payload) => payload,
                        Err(_) => return RouteOutcome::Error(RocketStatus::InternalServerError),
                    },
                    Ok(Either::B(payload)) => payload,
                    Err(_) => return RouteOutcome::Error(RocketStatus::InternalServerError),
                };
                RouteOutcome::Success(build_response(&output))
            }
            RouteKind::Static(payload) => RouteOutcome::Success(build_response(payload.as_ref())),
            RouteKind::File { path, _cache } => match rocket::tokio::fs::read(path).await {
                Ok(bytes) => {
                    let mut builder = RocketResponse::build();
                    builder.sized_body(bytes.len(), Cursor::new(bytes));
                    RouteOutcome::Success(builder.finalize())
                }
                Err(_) => RouteOutcome::Error(RocketStatus::NotFound),
            },
        }
    }
}

async fn build_request(request: &Request<'_>, data: Data<'_>, pattern: &str) -> RequestInput {
    let method = request.method().as_str().to_string();
    let uri = request.uri();
    let path = uri.path().as_str().to_string();
    let query = uri.query().map(|q| q.to_string()).unwrap_or_default();
    let headers = request
        .headers()
        .iter()
        .map(|h| Header {
            name: h.name.as_str().to_string(),
            value: h.value().to_string(),
        })
        .collect();

    let body = match data.open(ByteUnit::Megabyte(1)).into_bytes().await {
        Ok(capped) => Some(Buffer::from(capped.value)),
        Err(_) => None,
    };

    let params = extract_params(pattern, &path);

    RequestInput {
        method,
        path,
        query,
        headers,
        body,
        params,
    }
}

fn extract_params(pattern: &str, path: &str) -> HashMap<String, String> {
    let p_segs: Vec<&str> = pattern
        .trim_start_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();
    let a_segs: Vec<&str> = path
        .trim_start_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();
    let mut params = HashMap::new();
    for (idx, seg) in p_segs.iter().enumerate() {
        let name = if let Some(inner) = seg.strip_prefix(':') {
            Some(inner)
        } else if let Some(inner) = seg.strip_prefix('<') {
            inner
                .strip_suffix("..>")
                .or_else(|| inner.strip_suffix('>'))
        } else {
            None
        };

        if let Some(name) = name {
            if name != "_" {
                if seg.contains("..>") {
                    if idx < a_segs.len() {
                        let trailing = a_segs[idx..].join("/");
                        params.insert(name.to_string(), trailing);
                    }
                    break;
                } else if let Some(v) = a_segs.get(idx) {
                    params.insert(name.to_string(), (*v).to_string());
                }
            }
        }
    }
    params
}

fn translate_pattern(pattern: &str) -> String {
    let mut out = String::with_capacity(pattern.len() + 1);
    if !pattern.starts_with('/') {
        out.push('/');
    }
    let mut chars = pattern.chars().peekable();
    while let Some(c) = chars.next() {
        if c == ':' {
            let mut name = String::new();
            while let Some(&nc) = chars.peek() {
                if nc.is_alphanumeric() || nc == '_' {
                    name.push(nc);
                    chars.next();
                } else {
                    break;
                }
            }
            if name.is_empty() {
                out.push(':');
            } else {
                out.push_str(&format!("<{name}>"));
            }
        } else if c == '*' {
            out.push_str("<_..>");
        } else {
            out.push(c);
        }
    }
    out
}

fn build_response(payload: &ResponsePayload) -> RocketResponse<'static> {
    let mut builder = RocketResponse::build();
    if let Some(status) = RocketStatus::from_code(payload.status as u16) {
        builder.status(status);
    }
    for h in &payload.headers {
        builder.header(RocketHeader::new(h.name.clone(), h.value.clone()));
    }
    if let Some(body) = &payload.body {
        let bytes: Vec<u8> = body.to_vec();
        builder.sized_body(bytes.len(), Cursor::new(bytes));
    }
    builder.finalize()
}

const ALL_METHODS: &[Method] = &[
    Method::Get,
    Method::Post,
    Method::Put,
    Method::Delete,
    Method::Patch,
    Method::Head,
    Method::Options,
];

#[allow(deprecated)]
fn create_handler_tsfn(env: &Env, handler: Unknown) -> napi::Result<HandlerTsfn> {
    let wrapper_factory: Unknown = env.run_script("(f) => (err, req) => f(err ? err : (Array.isArray(req) ? req[0] : req))")?;
    let wrapper_factory_fn = to_js_function(wrapper_factory);
    let handler_fn = to_js_function(handler);
    let wrapped_val: Unknown = wrapper_factory_fn.call(None, &[handler_fn])?;
    let f = to_js_function(wrapped_val);
    f.create_threadsafe_function::<RequestInput, (serde_json::Value,), Either<Promise<ResponsePayload>, ResponsePayload>, Status, _, true, false, 0>(
        |ctx| {
            let headers_json: Vec<serde_json::Value> = ctx
                .value
                .headers
                .into_iter()
                .map(|h| serde_json::json!({ "name": h.name, "value": h.value }))
                .collect();
            let body_json = ctx.value.body.map(|b| b.to_vec());
            let val = serde_json::json!({
                "method": ctx.value.method,
                "path": ctx.value.path,
                "query": ctx.value.query,
                "headers": headers_json,
                "body": body_json,
                "params": ctx.value.params,
            });
            Ok((val,))
        },
    )
}

#[allow(deprecated)]
fn mount_value<'env>(
    env: &Env,
    value: Unknown<'env>,
    pattern: &str,
    methods: &[Method],
    out: &mut Vec<Route>,
) -> napi::Result<()> {
    if value.get_type()? == ValueType::Function {
        let tsfn: HandlerTsfn = create_handler_tsfn(env, value)?;
        push_kind(RouteKind::Fn(Arc::new(tsfn)), pattern, methods, out);
        return Ok(());
    }

    if value.get_type()? == ValueType::Object {
        let obj: JsObject = unsafe { value.clone().cast()? };
        if let Ok(path) = obj.get_named_property::<String>("path") {
            push_kind(
                RouteKind::File {
                    path,
                    _cache: Arc::new(Mutex::new(None)),
                },
                pattern,
                methods,
                out,
            );
            return Ok(());
        }
        if let Ok(html) = obj.get_named_property::<String>("html") {
            push_kind(
                RouteKind::Static(Arc::new(ResponsePayload {
                    status: 200,
                    headers: vec![Header {
                        name: "Content-Type".to_string(),
                        value: "text/html; charset=utf-8".to_string(),
                    }],
                    body: Some(Buffer::from(html.into_bytes())),
                })),
                pattern,
                methods,
                out,
            );
            return Ok(());
        }
        if let Ok(status) = obj.get_named_property::<u32>("status") {
            let headers: Vec<Header> = obj.get_named_property("headers").unwrap_or_default();
            let body: Option<Buffer> = obj.get_named_property("body").unwrap_or(None);
            push_kind(
                RouteKind::Static(Arc::new(ResponsePayload {
                    status,
                    headers,
                    body,
                })),
                pattern,
                methods,
                out,
            );
            return Ok(());
        }

        let map: HashMap<String, Unknown<'env>> = unsafe { value.cast()? };
        for (method, method_value) in map {
            if let Ok(m) = method.parse::<Method>() {
                mount_value(env, method_value, pattern, &[m], out)?;
            }
        }
    }
    Ok(())
}

fn push_kind(kind: RouteKind, pattern: &str, methods: &[Method], out: &mut Vec<Route>) {
    let handler = JsHandler {
        kind: Arc::new(kind),
        pattern: pattern.to_string(),
    };
    let rocket_pattern = translate_pattern(pattern);
    for m in methods {
        out.push(Route::new(*m, &rocket_pattern, handler.clone()));
    }
}

static RT: OnceLock<rocket::tokio::runtime::Runtime> = OnceLock::new();

fn tokio_runtime() -> &'static rocket::tokio::runtime::Runtime {
    RT.get_or_init(|| {
        rocket::tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to initialize Tokio runtime")
    })
}

#[napi]
pub fn serve(
    env: Env,
    routes: Option<Unknown<'_>>,
    fetch: Option<Unknown<'_>>,
    options: Option<ServeOptions>,
) -> napi::Result<Server> {
    let mut routes_val = routes;
    let mut fetch_val = fetch;
    let mut options = options.unwrap_or_else(|| ServeOptions {
        port: None,
        hostname: None,
        unix: None,
    });

    if let Some(ref arg1) = routes_val {
        if arg1.get_type()? == ValueType::Object {
            let obj: JsObject = unsafe { arg1.clone().cast()? };
            if obj.has_named_property("port")?
                || obj.has_named_property("hostname")?
                || obj.has_named_property("routes")?
                || obj.has_named_property("fetch")?
            {
                if options.port.is_none() {
                    options.port = obj.get_named_property("port").ok();
                }
                if options.hostname.is_none() {
                    options.hostname = obj.get_named_property("hostname").ok();
                }
                if options.unix.is_none() {
                    options.unix = obj.get_named_property("unix").ok();
                }
                if obj.has_named_property("routes")? {
                    routes_val = obj.get_named_property("routes").ok();
                }
                if fetch_val.is_none() && obj.has_named_property("fetch")? {
                    fetch_val = obj.get_named_property("fetch").ok();
                }
            }
        }
    }

    let env_port = std::env::var("BUN_PORT")
        .or_else(|_| std::env::var("PORT"))
        .or_else(|_| std::env::var("NODE_PORT"))
        .ok()
        .and_then(|p| p.parse::<u32>().ok());
    let port = options.port.or(env_port).unwrap_or(3000) as u16;
    let hostname = options
        .hostname
        .clone()
        .unwrap_or_else(|| "0.0.0.0".to_string());
    let address: IpAddr = match hostname.as_str() {
        "localhost" => IpAddr::V4(Ipv4Addr::LOCALHOST),
        other => other
            .parse()
            .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
    };
    let url = format!("http://{}:{}", hostname, port);

    let mut out: Vec<Route> = Vec::new();
    if let Some(table) = routes_val {
        let table: JsObject = unsafe { table.cast()? };
        let names = table.get_property_names()?;
        let len = names.get_array_length_unchecked()?;
        for i in 0..len {
            let pattern: String = names.get_element(i)?;
            let value: Unknown<'_> = table.get_named_property(&pattern)?;
            mount_value(&env, value, &pattern, ALL_METHODS, &mut out)?;
        }
    }

    if let Some(fetch) = fetch_val {
        let tsfn: HandlerTsfn = create_handler_tsfn(&env, fetch)?;
        let tsfn = Arc::new(tsfn);
        for m in ALL_METHODS {
            out.push(Route::ranked(10, *m, "/<_..>", JsHandler {
                kind: Arc::new(RouteKind::Fn(tsfn.clone())),
                pattern: "/<_..>".to_string(),
            }));
        }
    }

    let mut config = Config::default();
    config.port = port;
    config.address = address;

    let shutdown_slot: Arc<Mutex<Option<Shutdown>>> = Arc::new(Mutex::new(None));
    let shutdown_slot_clone = shutdown_slot.clone();

    let rocket = rocket::build()
        .configure(config)
        .mount("/", out)
        .attach(AdHoc::on_liftoff("capture-shutdown", move |orbit| {
            Box::pin(async move {
                *shutdown_slot_clone.lock().unwrap() = Some(orbit.shutdown());
            })
        }));
    tokio_runtime().spawn(async move {
        let _ = rocket.launch().await;
    });

    Ok(Server {
        url,
        port: port as u32,
        shutdown: shutdown_slot,
    })
}

#[napi(object)]
pub struct ServeOptions {
    pub port: Option<u32>,
    pub hostname: Option<String>,
    pub unix: Option<String>,
}

#[napi]
pub struct Server {
    pub url: String,
    pub port: u32,
    shutdown: Arc<Mutex<Option<Shutdown>>>,
}

#[napi]
impl Server {
    #[napi]
    pub fn stop(&self) -> napi::Result<()> {
        if let Some(shutdown) = self.shutdown.lock().unwrap().take() {
            shutdown.notify();
        }
        Ok(())
    }
}

static ROUTES: OnceLock<Mutex<Vec<Route>>> = OnceLock::new();

fn routes() -> &'static Mutex<Vec<Route>> {
    ROUTES.get_or_init(|| Mutex::new(Vec::new()))
}

#[napi]
pub fn register_route(env: Env, method: String, path: String, handler: Unknown<'_>) -> napi::Result<()> {
    let tsfn: HandlerTsfn = create_handler_tsfn(&env, handler)?;
    let m: Method = method
        .parse()
        .map_err(|_| napi::Error::new(Status::GenericFailure, format!("invalid method: {method}")))?;
    let mut routes = routes().lock().unwrap();
    push_kind(RouteKind::Fn(Arc::new(tsfn)), &path, &[m], &mut routes);
    Ok(())
}

#[napi]
pub async fn launch() -> napi::Result<()> {
    let routes = std::mem::take(&mut *routes().lock().unwrap());
    rocket::build()
        .mount("/", routes)
        .launch()
        .await
        .map(|_orbit| ())
        .map_err(|e| napi::Error::new(Status::GenericFailure, e.to_string()))
}

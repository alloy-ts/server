//! # NAPI bindings that bridge Rocket's runtime to TypeScript handlers.
//!
//! Rocket's real codegen (`#[get]`, `routes!`, `derive(..)`, `uri!`) are Rust
//! *procedural macros* and cannot cross the N-API boundary. This module is the
//! NAPI equivalent of the *responsibilities* those macros perform:
//!
//! * `#[get("/p")]` / `routes!` -> [`serve`] + the route-table parser, which at
//!   runtime build the same `&[Route]`-equivalent that Rocket's codegen emits,
//!   except the handler bodies live in JavaScript.
//! * `uri!` type-checked construction -> the TS-side `uri()` builder (see the
//!   codegen layer); this module only carries the request/response wire types.

use std::collections::HashMap;
use std::io::Cursor;
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use std::sync::{Arc, Mutex, OnceLock};

use napi::bindgen_prelude::{Buffer, Function, JsObjectValue, Object, Promise, Status, Unknown, ValueType};
use napi::threadsafe_function::ThreadsafeFunction;
use napi_derive::napi;

use rocket::config::Config;
use rocket::data::ByteUnit;
use rocket::http::{Header, Method, Status as RocketStatus};
use rocket::request::Request;
use rocket::response::Response as RocketResponse;
use rocket::route::{Handler, Outcome, Route};
use rocket::{Data, Shutdown};

/// Wire type sent from Rust -> JS for every request.
#[napi(object)]
pub struct RequestInput {
    pub method: String,
    pub path: String,
    pub query: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Buffer>,
    pub params: HashMap<String, String>,
}

/// Init passed to the `Response` constructor / static helpers.
#[napi(object)]
#[derive(Clone)]
pub struct ResponseInit {
    pub status: Option<u32>,
    pub headers: Option<HashMap<String, String>>,
}

/// The `Response` value a JS handler returns (or a static route value).
#[napi]
pub struct Response {
    status: u32,
    headers: HashMap<String, String>,
    body: Option<Buffer>,
}

#[napi]
impl Response {
    #[napi(getter)]
    pub fn status(&self) -> u32 {
        self.status
    }

    #[napi(getter)]
    pub fn headers(&self) -> HashMap<String, String> {
        self.headers.clone()
    }

    #[napi(getter)]
    pub fn body(&self) -> Option<Buffer> {
        self.body
            .as_ref()
            .map(|b| Buffer::from(b.as_ref().to_vec()))
    }

    #[napi(constructor)]
    pub fn new(body: Option<Unknown>, init: Option<ResponseInit>) -> napi::Result<Self> {
        let body = match body {
            None => None,
            Some(v) => match v.get_type()? {
                ValueType::String => {
                    let s: String = unsafe { v.cast() }?;
                    Some(Buffer::from(s.into_bytes()))
                }
                _ => {
                    let b: Buffer = unsafe { v.cast() }?;
                    Some(b)
                }
            },
        };
        let init = init.unwrap_or(ResponseInit {
            status: None,
            headers: None,
        });
        Ok(Response {
            status: init.status.unwrap_or(200),
            headers: init.headers.unwrap_or_default(),
            body,
        })
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
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        Ok(Response {
            status: init.status.unwrap_or(200),
            headers,
            body: Some(Buffer::from(bytes)),
        })
    }

    #[napi]
    pub fn redirect(url: String, status: Option<u32>) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Location".to_string(), url);
        Response {
            status: status.unwrap_or(302),
            headers,
            body: None,
        }
    }
}

/// Wire type read back *from* JS.
#[napi(object)]
pub struct ResponsePayload {
    pub status: Option<u32>,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<Buffer>,
}

impl Clone for ResponsePayload {
    fn clone(&self) -> Self {
        Self {
            status: self.status,
            headers: self.headers.clone(),
            body: self.body.as_ref().map(|b| Buffer::from(b.as_ref().to_vec())),
        }
    }
}

/// Lazily-served static file marker returned by [`file`].
#[napi(object)]
#[derive(Clone)]
pub struct StaticFile {
    pub path: String,
}

/// `Alloy.file(path)` — marks a route value as a file served from disk.
#[napi]
pub fn file(path: String) -> StaticFile {
    StaticFile { path }
}

type RouteHandlerFn =
    ThreadsafeFunction<RequestInput, Promise<ResponsePayload>, RequestInput, Status, false, false, 0>;

#[derive(Clone)]
enum RouteHandler {
    Dynamic(Arc<RouteHandlerFn>),
    Static(ResponsePayload),
    File(String),
}

#[derive(Clone)]
struct JsHandler {
    handler: RouteHandler,
    pattern: String,
}

#[rocket::async_trait]
impl Handler for JsHandler {
    async fn handle<'r>(&self, request: &'r Request<'_>, data: Data<'r>) -> Outcome<'r> {
        match &self.handler {
            RouteHandler::Dynamic(tsfn) => {
                let input = build_request(request, data, &self.pattern).await;
                let output = match tsfn.call_async(input).await {
                    Ok(promise) => match promise.await {
                        Ok(out) => out,
                        Err(_) => return Outcome::Error(RocketStatus::InternalServerError),
                    },
                    Err(_) => return Outcome::Error(RocketStatus::InternalServerError),
                };
                Outcome::Success(build_response(output))
            }
            RouteHandler::Static(resp) => Outcome::Success(build_response(resp.clone())),
            RouteHandler::File(path) => match rocket::tokio::fs::read(path).await {
                Ok(bytes) => {
                    let mut builder = RocketResponse::build();
                    builder.sized_body(bytes.len(), Cursor::new(bytes));
                    Outcome::Success(builder.finalize())
                }
                Err(_) => Outcome::Error(RocketStatus::NotFound),
            },
        }
    }
}

async fn build_request(request: &Request<'_>, data: Data<'_>, pattern: &str) -> RequestInput {
    let method = request.method().as_str().to_string();
    let uri = request.uri();
    let path = uri.path().to_string();
    let query = uri.query().map(|q| q.to_string()).unwrap_or_default();
    let headers = request
        .headers()
        .iter()
        .map(|h| (h.name().as_str().to_string(), h.value().to_string()))
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
        if let Some(inner) = seg.strip_prefix('<') {
            let name = inner
                .strip_suffix("..>")
                .or_else(|| inner.strip_suffix('>'));
            if let Some(name) = name {
                if name != "_" {
                    if let Some(v) = a_segs.get(idx) {
                        params.insert(name.to_string(), (*v).to_string());
                    }
                }
            }
        }
    }
    params
}

fn translate_pattern(p: &str) -> String {
    let mut out = String::with_capacity(p.len() + 4);
    let bytes = p.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b':' {
            out.push('<');
            i += 1;
            while i < bytes.len() && bytes[i] != b'/' {
                out.push(bytes[i] as char);
                i += 1;
            }
            out.push('>');
        } else if bytes[i] == b'*' {
            out.push_str("<_..>");
            i += 1;
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}

fn build_response(out: ResponsePayload) -> RocketResponse<'static> {
    let mut builder = RocketResponse::build();
    if let Some(status) = RocketStatus::from_code(out.status.unwrap_or(200) as u16) {
        builder.status(status);
    }
    for (name, value) in out.headers.unwrap_or_default() {
        builder.header(Header::new(name, value));
    }
    if let Some(body) = out.body {
        let bytes: Vec<u8> = body.as_ref().to_vec();
        builder.sized_body(bytes.len(), Cursor::new(bytes));
    }
    builder.finalize()
}

#[napi(object)]
pub struct ServeOptions<'a> {
    pub port: Option<u32>,
    pub hostname: Option<String>,
    pub unix: Option<String>,
    pub routes: Option<Object<'a>>,
    pub fetch: Option<Unknown<'a>>,
}

const ALL_METHODS: &[&str] = &["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];

fn collect_routes(value: Unknown) -> napi::Result<Vec<(Option<String>, RouteHandler)>> {
    match value.get_type()? {
        ValueType::Function => {
            let f: Function<'_, RequestInput, Promise<ResponsePayload>> = unsafe { value.cast() }?;
            let tsfn: RouteHandlerFn = f.build_threadsafe_function().build()?;
            Ok(vec![(None, RouteHandler::Dynamic(Arc::new(tsfn)))])
        }
        ValueType::Object => {
            if let Ok(file) = unsafe { value.cast::<StaticFile>() } {
                return Ok(vec![(None, RouteHandler::File(file.path))]);
            }
            if let Ok(resp) = unsafe { value.cast::<ResponsePayload>() } {
                return Ok(vec![(None, RouteHandler::Static(resp))]);
            }
            let obj: Object = unsafe { value.cast() }?;
            let names = obj.get_property_names()?;
            let len = names.get_array_length()?;
            let mut out = Vec::new();
            for i in 0..len {
                let method: String = names.get_element(i)?;
                let v: Unknown = obj.get_named_property(&method)?;
                for (_, handler) in collect_routes(v)? {
                    out.push((Some(method.to_uppercase()), handler));
                }
            }
            Ok(out)
        }
        _ => Err(napi::Error::new(
            Status::InvalidArg,
            "route value must be a function, Response, file(), or a method map",
        )),
    }
}

static SHUTDOWN: OnceLock<Mutex<Option<Shutdown>>> = OnceLock::new();

#[napi]
pub fn serve(options: ServeOptions<'_>) -> napi::Result<Server> {
    let port = options.port.unwrap_or(3000) as u16;
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

    let mut rocket_routes: Vec<Route> = Vec::new();
    if let Some(routes) = options.routes {
        let names = routes.get_property_names()?;
        let len = names.get_array_length()?;
        for i in 0..len {
            let pattern: String = names.get_element(i)?;
            let value: Unknown = routes.get_named_property(&pattern)?;
            for (method_opt, handler) in collect_routes(value)? {
                let methods: Vec<String> = method_opt
                    .map(|m| vec![m])
                    .unwrap_or_else(|| ALL_METHODS.iter().map(|s| s.to_string()).collect());
                for method in methods {
                    let m = Method::from_str(&method).map_err(|()| {
                        napi::Error::new(Status::InvalidArg, format!("invalid method: {method}"))
                    })?;
                    let translated = translate_pattern(&pattern);
                    rocket_routes.push(Route::new(
                        m,
                        &translated,
                        JsHandler {
                            handler: handler.clone(),
                            pattern: translated.clone(),
                        },
                    ));
                }
            }
        }
    }

    if let Some(fetch_val) = options.fetch {
        let f: Function<'_, RequestInput, Promise<ResponsePayload>> = unsafe { fetch_val.cast() }?;
        let tsfn: RouteHandlerFn = f.build_threadsafe_function().build()?;
        let tsfn = Arc::new(tsfn);
        for m in ALL_METHODS {
            rocket_routes.push(Route::new(
                Method::from_str(m).unwrap(),
                "<_..>",
                JsHandler {
                    handler: RouteHandler::Dynamic(tsfn.clone()),
                    pattern: "<_..>".to_string(),
                },
            ));
        }
    }

    let mut config = Config::default();
    config.port = port;
    config.address = address;

    let rocket = rocket::build().configure(config).mount("/", rocket_routes);

    std::thread::spawn(move || {
        let rt = rocket::tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build();
        if let Ok(rt) = rt {
            rt.block_on(async move {
                if let Ok(ignited) = rocket.ignite().await {
                    let shutdown = ignited.shutdown();
                    *SHUTDOWN.get_or_init(|| Mutex::new(None)).lock().unwrap() = Some(shutdown);
                    let _ = ignited.launch().await;
                }
            });
        }
    });

    Ok(Server {
        url,
        port: port as u32,
    })
}

#[napi]
pub struct Server {
    pub url: String,
    pub port: u32,
}

#[napi]
impl Server {
    #[napi]
    pub fn stop(&self) -> napi::Result<()> {
        if let Some(shutdown) = SHUTDOWN.get_or_init(|| Mutex::new(None)).lock().unwrap().take() {
            shutdown.notify();
        }
        Ok(())
    }
}

struct RouteSpec {
    method: String,
    path: String,
    handler: RouteHandler,
}

static ROUTES: OnceLock<Mutex<Vec<RouteSpec>>> = OnceLock::new();

fn routes() -> &'static Mutex<Vec<RouteSpec>> {
    ROUTES.get_or_init(|| Mutex::new(Vec::new()))
}

#[napi]
pub fn register_route(method: String, path: String, handler: Unknown) -> napi::Result<()> {
    let f: Function<'_, RequestInput, Promise<ResponsePayload>> = unsafe { handler.cast() }?;
    let tsfn: RouteHandlerFn = f.build_threadsafe_function().build()?;
    routes()
        .lock()
        .unwrap()
        .push(RouteSpec {
            method,
            path,
            handler: RouteHandler::Dynamic(Arc::new(tsfn)),
        });
    Ok(())
}

#[napi]
pub async fn launch() -> napi::Result<()> {
    let specs = std::mem::take(&mut *routes().lock().unwrap());
    let mut rocket_routes: Vec<Route> = Vec::with_capacity(specs.len());
    for spec in specs {
        let method = Method::from_str(&spec.method).map_err(|()| {
            napi::Error::new(Status::InvalidArg, format!("invalid method: {}", spec.method))
        })?;
        let translated = translate_pattern(&spec.path);
        rocket_routes.push(Route::new(
            method,
            &translated,
            JsHandler {
                handler: spec.handler,
                pattern: translated.clone(),
            },
        ));
    }

    let ignited = rocket::build()
        .mount("/", rocket_routes)
        .ignite()
        .await
        .map_err(|e| napi::Error::new(Status::GenericFailure, e.to_string()))?;

    let shutdown = ignited.shutdown();
    *SHUTDOWN.get_or_init(|| Mutex::new(None)).lock().unwrap() = Some(shutdown);

    ignited
        .launch()
        .await
        .map(|_orbit| ())
        .map_err(|e| napi::Error::new(Status::GenericFailure, e.to_string()))
}

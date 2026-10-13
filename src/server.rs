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
//!
//! Each registered JS handler is wrapped in a [`rocket::route::Handler`] whose
//! `handle` calls the JS function through a [`napi::threadsafe_function`] (so it
//! can run on any Rocket worker thread) and builds a [`rocket::Response`] from
//! the JS return value, mirroring Rocket's generated-handler semantics.

use std::collections::HashMap;
use std::io::Cursor;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Arc, Mutex, OnceLock};

use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
#[allow(deprecated)]
use napi::{Env, JsFunction, Unknown, ValueType};
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
type HandlerTsfn =
    ThreadsafeFunction<RequestInput, Promise<Response>, RequestInput, Status, false, false, 0>;

/// Convert a `napi::Unknown` into a `napi::JsFunction`.
#[allow(deprecated)]
fn to_js_function(handler: Unknown) -> JsFunction {
    debug_assert_eq!(
        std::mem::size_of::<Unknown>(),
        std::mem::size_of::<JsFunction>()
    );
    unsafe { std::mem::transmute_copy::<Unknown, JsFunction>(&handler) }
}

/// Create a threadsafe function for JS route handlers, wrapping the function
/// in `async (req) => handler(req)` so both sync and async JS handlers work.
#[allow(deprecated)]
fn create_handler_tsfn(env: &Env, handler: Unknown) -> napi::Result<HandlerTsfn> {
    let factory_unknown: Unknown = env.run_script("handler => async (req) => handler(req)")?;
    let factory = to_js_function(factory_unknown);
    let handler_func = to_js_function(handler);
    let async_handler_unknown = factory.call(None, &[handler_func])?;
    let async_handler = to_js_function(async_handler_unknown);
    async_handler.create_threadsafe_function::<RequestInput, RequestInput, Promise<Response>, Status, _, false, false, 0>(
        |ctx| Ok(ctx.value),
    )
}

/// Wire type sent from Rust -> JS for every request.
#[napi(object)]
pub struct RequestInput {
    pub method: String,
    pub path: String,
    pub query: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub params: HashMap<String, String>,
}

/// Init passed to the `Response` constructors / helpers.
#[napi(object)]
pub struct ResponseInit {
    pub status: Option<u32>,
    pub headers: Option<HashMap<String, String>>,
}

/// Plain response object shape carried across the N-API boundary.
#[napi(object)]
#[derive(Clone, Debug)]
pub struct Response {
    pub status: u32,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

/// Free function helper `response(body, init)`.
#[napi]
pub fn response(
    body: Option<Either<String, Buffer>>,
    init: Option<ResponseInit>,
) -> Response {
    let init = init.unwrap_or(ResponseInit {
        status: None,
        headers: None,
    });
    let body_bytes = body.map(|b| match b {
        Either::A(s) => s.into_bytes(),
        Either::B(buf) => buf.to_vec(),
    });
    Response {
        status: init.status.unwrap_or(200),
        headers: init.headers.unwrap_or_default(),
        body: body_bytes,
    }
}

/// Free function helper `responseJson(data, init)`.
#[napi]
pub fn response_json(
    data: serde_json::Value,
    init: Option<ResponseInit>,
) -> napi::Result<Response> {
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
        body: Some(bytes),
    })
}

/// Free function helper `responseRedirect(url, status)`.
#[napi]
pub fn response_redirect(url: String, status: Option<u32>) -> Response {
    let mut headers = HashMap::new();
    headers.insert("Location".to_string(), url);
    Response {
        status: status.unwrap_or(302),
        headers,
        body: None,
    }
}

/// Bun / Alloy compatible `Response` class backing `new Response()`,
/// `Response.json()`, and `Response.redirect()`.
#[napi(js_name = "Response")]
#[derive(Clone, Default)]
pub struct ResponseClass {
    pub status: u32,
    pub body: Option<Vec<u8>>,
    headers_map: HashMap<String, String>,
}

#[napi]
impl ResponseClass {
    #[napi(constructor)]
    pub fn new(body: Option<Either<String, Buffer>>, init: Option<ResponseInit>) -> Self {
        let resp = response(body, init);
        ResponseClass {
            status: resp.status,
            body: resp.body,
            headers_map: resp.headers,
        }
    }

    #[napi(factory)]
    pub fn json(data: serde_json::Value, init: Option<ResponseInit>) -> napi::Result<Self> {
        let resp = response_json(data, init)?;
        Ok(ResponseClass {
            status: resp.status,
            body: resp.body,
            headers_map: resp.headers,
        })
    }

    #[napi(factory)]
    pub fn redirect(url: String, status: Option<u32>) -> Self {
        let resp = response_redirect(url, status);
        ResponseClass {
            status: resp.status,
            body: resp.body,
            headers_map: resp.headers,
        }
    }

    #[napi(getter, js_name = "headers")]
    pub fn headers(&self) -> HashMap<String, String> {
        self.headers_map.clone()
    }
}

/// Lazily-served static file marker returned by [`file`].
#[napi(object)]
pub struct StaticFile {
    pub path: String,
}

/// `file(path)` — marks a route value as a file served from disk.
#[napi]
pub fn file(path: String) -> StaticFile {
    StaticFile { path }
}

/// The resolved intent of one route entry after parsing the JS `routes` table.
#[derive(Clone)]
enum RouteKind {
    Fn(Arc<HandlerTsfn>),
    Static(Arc<Response>),
    File { path: String },
}

/// A Rocket `Handler` that dispatches to one of the three [`RouteKind`] forms.
#[derive(Clone)]
struct JsHandler(Arc<RouteKind>);

#[rocket::async_trait]
impl Handler for JsHandler {
    async fn handle<'r>(&self, request: &'r Request<'_>, data: Data<'r>) -> RouteOutcome<'r> {
        let pattern = request
            .route()
            .map(|r| r.uri.to_string())
            .unwrap_or_default();
        match &*self.0 {
            RouteKind::Fn(tsfn) => {
                let input = build_request(request, data, &pattern).await;
                let output = match tsfn.call_async(input).await {
                    Ok(promise) => match promise.await {
                        Ok(payload) => payload,
                        Err(_) => return RouteOutcome::Error(RocketStatus::InternalServerError),
                    },
                    Err(_) => return RouteOutcome::Error(RocketStatus::InternalServerError),
                };
                RouteOutcome::Success(build_response(&output))
            }
            RouteKind::Static(payload) => RouteOutcome::Success(build_response(payload.as_ref())),
            RouteKind::File { path } => match rocket::tokio::fs::read(path).await {
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
    let path = uri.path().to_string();
    let query = uri.query().map(|q| q.to_string()).unwrap_or_default();
    let headers = request
        .headers()
        .iter()
        .map(|h| (h.name.as_str().to_string(), h.value().to_string()))
        .collect();

    let body = match data.open(ByteUnit::Megabyte(1)).into_bytes().await {
        Ok(capped) => Some(capped.value),
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
            let is_multi = inner.ends_with("..>");
            let name = inner
                .strip_suffix("..>")
                .or_else(|| inner.strip_suffix('>'));
            if let Some(name) = name {
                if is_multi {
                    if idx < a_segs.len() {
                        let rest = a_segs[idx..].join("/");
                        params.insert(name.to_string(), rest);
                    }
                } else if let Some(v) = a_segs.get(idx) {
                    params.insert(name.to_string(), (*v).to_string());
                }
            }
        }
    }
    params
}

fn translate_pattern(pattern: &str) -> String {
    let mut out = String::with_capacity(pattern.len() + 2);
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

fn build_response(payload: &Response) -> RocketResponse<'static> {
    let mut builder = RocketResponse::build();
    if let Some(status) = RocketStatus::from_code(payload.status as u16) {
        builder.status(status);
    }
    for (name, value) in &payload.headers {
        builder.header(RocketHeader::new(name.clone(), value.clone()));
    }
    if let Some(body) = &payload.body {
        let bytes = body.clone();
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

fn is_response_object(v: &Unknown) -> bool {
    if v.get_type().ok() == Some(ValueType::Object) {
        if let Ok(obj) = unsafe { v.clone().cast::<Object>() } {
            return obj.has_named_property("status").unwrap_or(false);
        }
    }
    false
}

#[allow(deprecated)]
fn mount_value<'env>(
    env: &'env Env,
    value: Unknown<'env>,
    pattern: &str,
    methods: &[Method],
    out: &mut Vec<Route>,
) -> napi::Result<()> {
    match value.get_type()? {
        ValueType::Function => {
            let tsfn = create_handler_tsfn(env, value)?;
            push_kind(RouteKind::Fn(Arc::new(tsfn)), pattern, methods, out);
        }
        ValueType::Object => {
            let v = value.clone();
            if let Ok(fr) = unsafe { v.clone().cast::<StaticFile>() } {
                push_kind(RouteKind::File { path: fr.path }, pattern, methods, out);
            } else if is_response_object(&v) {
                let obj: Object = unsafe { v.cast()? };
                let status: u32 = obj.get_named_property("status").unwrap_or(200);
                let headers: HashMap<String, String> =
                    obj.get_named_property("headers").unwrap_or_default();
                let body: Option<Vec<u8>> = obj.get_named_property("body").ok().flatten();
                let r = Response {
                    status,
                    headers,
                    body,
                };
                push_kind(RouteKind::Static(Arc::new(r)), pattern, methods, out);
            } else {
                let map: HashMap<String, Unknown<'env>> = unsafe { v.cast()? };
                for (method, method_value) in map {
                    let m: Method = method
                        .parse()
                        .map_err(|_| {
                            napi::Error::new(
                                Status::GenericFailure,
                                format!("invalid method: {method}"),
                            )
                        })?;
                    mount_value(env, method_value, pattern, &[m], out)?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn push_kind(kind: RouteKind, pattern: &str, methods: &[Method], out: &mut Vec<Route>) {
    let handler = JsHandler(Arc::new(kind));
    let rocket_pattern = translate_pattern(pattern);
    let is_wildcard = rocket_pattern.contains("<_..>");
    let seg_count = pattern.split('/').filter(|s| !s.is_empty()).count();
    for m in methods {
        let mut route = Route::new(*m, &rocket_pattern, handler.clone());
        if is_wildcard {
            if pattern == "/<_..>" || pattern == "/*" || pattern == "<_..>" {
                route.rank = 100;
            } else {
                route.rank = (10 - seg_count as isize).max(1) as isize;
            }
        }
        out.push(route);
    }
}

type ShutdownSlot = Arc<Mutex<Option<Shutdown>>>;

/// NAPI equivalent of `Alloy.serve({ routes, fetch, port, hostname, unix })`
/// and Bun.serve compatible single-object configuration.
#[napi]
pub fn serve(
    env: Env,
    routes_or_options: Option<Unknown<'_>>,
    fetch: Option<Unknown<'_>>,
    options: Option<ServeOptions>,
) -> napi::Result<Server> {
    let mut routes_obj: Option<Unknown<'_>> = None;
    let mut fetch_fn: Option<Unknown<'_>> = fetch;
    let mut port_opt: Option<u32> = options.as_ref().and_then(|o| o.port);
    let mut hostname_opt: Option<String> = options.as_ref().and_then(|o| o.hostname.clone());
    let mut unix_opt: Option<String> = options.as_ref().and_then(|o| o.unix.clone());

    if let Some(first_arg) = routes_or_options {
        if first_arg.get_type()? == ValueType::Object {
            let obj: Object = unsafe { first_arg.clone().cast()? };
            let has_routes = obj.has_named_property("routes")?;
            let has_fetch = obj.has_named_property("fetch")?;
            let has_port = obj.has_named_property("port")?;
            let has_hostname = obj.has_named_property("hostname")?;
            let has_unix = obj.has_named_property("unix")?;

            if has_routes || has_fetch || has_port || has_hostname || has_unix {
                if has_routes {
                    routes_obj = Some(obj.get_named_property("routes")?);
                }
                if has_fetch {
                    fetch_fn = Some(obj.get_named_property("fetch")?);
                }
                if has_port && port_opt.is_none() {
                    let p: u32 = obj.get_named_property("port")?;
                    port_opt = Some(p);
                }
                if has_hostname && hostname_opt.is_none() {
                    let h: String = obj.get_named_property("hostname")?;
                    hostname_opt = Some(h);
                }
                if has_unix && unix_opt.is_none() {
                    let u: String = obj.get_named_property("unix")?;
                    unix_opt = Some(u);
                }
            } else {
                routes_obj = Some(first_arg);
            }
        }
    }

    let port = port_opt
        .or_else(|| {
            std::env::var("BUN_PORT")
                .ok()
                .and_then(|v| v.parse::<u32>().ok())
        })
        .or_else(|| {
            std::env::var("PORT")
                .ok()
                .and_then(|v| v.parse::<u32>().ok())
        })
        .or_else(|| {
            std::env::var("NODE_PORT")
                .ok()
                .and_then(|v| v.parse::<u32>().ok())
        })
        .unwrap_or(3000) as u16;

    let hostname = hostname_opt.unwrap_or_else(|| "0.0.0.0".to_string());
    let address: IpAddr = match hostname.as_str() {
        "localhost" => IpAddr::V4(Ipv4Addr::LOCALHOST),
        other => other
            .parse()
            .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
    };
    let url = if let Some(path) = unix_opt {
        format!("unix://{path}")
    } else {
        format!("http://{}:{}", hostname, port)
    };

    let mut out: Vec<Route> = Vec::new();

    // Include registered routes from `register_route`
    {
        if let Ok(mut reg) = routes_registry().lock() {
            out.append(&mut reg);
        }
    }

    if let Some(table) = routes_obj {
        let table: Object = unsafe { table.cast()? };
        let names = table.get_property_names()?;
        let len = names.get_array_length_unchecked()?;
        for i in 0..len {
            let pattern: String = names.get_element(i)?;
            let value: Unknown<'_> = table.get_named_property(&pattern)?;
            mount_value(&env, value, &pattern, ALL_METHODS, &mut out)?;
        }
    }

    // `fetch` fallback: a catch-all for every method on any unmatched path.
    if let Some(fetch) = fetch_fn {
        let tsfn = create_handler_tsfn(&env, fetch)?;
        let tsfn = Arc::new(tsfn);
        for m in ALL_METHODS {
            push_kind(RouteKind::Fn(tsfn.clone()), "/<_..>", &[*m], &mut out);
        }
    }

    let mut config = Config::default();
    config.port = port;
    config.address = address;

    let shutdown_slot: ShutdownSlot = Arc::new(Mutex::new(None));
    let capture_slot = shutdown_slot.clone();

    let rocket = rocket::build()
        .configure(config)
        .mount("/", out)
        .attach(AdHoc::on_liftoff("capture-shutdown", move |orbit| {
            let slot = capture_slot.clone();
            Box::pin(async move {
                *slot.lock().unwrap() = Some(orbit.shutdown());
            })
        }));

    std::thread::spawn(move || {
        if let Ok(rt) = rocket::tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
        {
            rt.block_on(async move {
                let _ = rocket.launch().await;
            });
        }
    });

    Ok(Server {
        url,
        port: port as u32,
        shutdown_slot,
    })
}

/// Options for [`serve`].
#[napi(object)]
pub struct ServeOptions {
    pub port: Option<u32>,
    pub hostname: Option<String>,
    pub unix: Option<String>,
}

/// Handle returned by [`serve`].
#[napi]
pub struct Server {
    pub url: String,
    pub port: u32,
    shutdown_slot: ShutdownSlot,
}

#[napi]
impl Server {
    /// Gracefully stop the server.
    #[napi]
    pub fn stop(&self) -> napi::Result<()> {
        if let Some(shutdown) = self.shutdown_slot.lock().unwrap().take() {
            shutdown.notify();
        }
        Ok(())
    }
}

/// Namespace exposing `Alloy.serve` and `Alloy.file`.
#[napi]
#[derive(Default)]
pub struct Alloy;

#[napi]
impl Alloy {
    #[napi(constructor)]
    pub fn new() -> Self {
        Alloy
    }

    #[napi]
    pub fn serve(
        env: Env,
        routes_or_options: Option<Unknown<'_>>,
        fetch: Option<Unknown<'_>>,
        options: Option<ServeOptions>,
    ) -> napi::Result<Server> {
        serve(env, routes_or_options, fetch, options)
    }

    #[napi]
    pub fn file(path: String) -> StaticFile {
        file(path)
    }
}

/// Rocket application builder mirroring `rocket::build()`.
#[napi]
#[derive(Default)]
pub struct App {
    server_handle: Arc<Mutex<Option<Server>>>,
}

#[napi]
impl App {
    #[napi(constructor)]
    pub fn new() -> Self {
        App {
            server_handle: Arc::new(Mutex::new(None)),
        }
    }

    #[napi]
    pub fn route(
        &self,
        env: Env,
        method: String,
        path: String,
        handler: Unknown<'_>,
    ) -> napi::Result<()> {
        register_route(env, method, path, handler)
    }

    #[napi]
    pub fn get(&self, env: Env, path: String, handler: Unknown<'_>) -> napi::Result<()> {
        self.route(env, "GET".to_string(), path, handler)
    }

    #[napi]
    pub fn post(&self, env: Env, path: String, handler: Unknown<'_>) -> napi::Result<()> {
        self.route(env, "POST".to_string(), path, handler)
    }

    #[napi]
    pub fn put(&self, env: Env, path: String, handler: Unknown<'_>) -> napi::Result<()> {
        self.route(env, "PUT".to_string(), path, handler)
    }

    #[napi]
    pub fn delete(&self, env: Env, path: String, handler: Unknown<'_>) -> napi::Result<()> {
        self.route(env, "DELETE".to_string(), path, handler)
    }

    #[napi]
    pub fn patch(&self, env: Env, path: String, handler: Unknown<'_>) -> napi::Result<()> {
        self.route(env, "PATCH".to_string(), path, handler)
    }

    #[napi]
    pub fn head(&self, env: Env, path: String, handler: Unknown<'_>) -> napi::Result<()> {
        self.route(env, "HEAD".to_string(), path, handler)
    }

    #[napi]
    pub fn options(&self, env: Env, path: String, handler: Unknown<'_>) -> napi::Result<()> {
        self.route(env, "OPTIONS".to_string(), path, handler)
    }

    #[napi]
    pub fn launch(&self, env: Env, options: Option<ServeOptions>) -> napi::Result<Server> {
        let s = serve(env, None, None, options)?;
        *self.server_handle.lock().unwrap() = Some(Server {
            url: s.url.clone(),
            port: s.port,
            shutdown_slot: s.shutdown_slot.clone(),
        });
        Ok(s)
    }

    #[napi]
    pub fn stop(&self) -> napi::Result<()> {
        if let Some(srv) = self.server_handle.lock().unwrap().as_ref() {
            srv.stop()?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Low-level registry API: `register_route`
// ---------------------------------------------------------------------------

static ROUTES_REGISTRY: OnceLock<Mutex<Vec<Route>>> = OnceLock::new();

fn routes_registry() -> &'static Mutex<Vec<Route>> {
    ROUTES_REGISTRY.get_or_init(|| Mutex::new(Vec::new()))
}

/// NAPI equivalent of `#[get("/p")] fn ..` + adding it to a `routes!` list.
#[napi]
pub fn register_route(
    env: Env,
    method: String,
    path: String,
    handler: Unknown<'_>,
) -> napi::Result<()> {
    let tsfn = create_handler_tsfn(&env, handler)?;
    let m: Method = method
        .parse()
        .map_err(|_| napi::Error::new(Status::GenericFailure, format!("invalid method: {method}")))?;
    let mut reg = routes_registry().lock().unwrap();
    push_kind(RouteKind::Fn(Arc::new(tsfn)), &path, &[m], &mut reg);
    Ok(())
}

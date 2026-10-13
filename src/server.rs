//! # NAPI bindings that bridge Rocket's runtime to TypeScript handlers.
//!
//! Rocket's real codegen (`#[get]`, `routes!`, `derive(..)`, `uri!`) are Rust
//! *procedural macros* and cannot cross the N-API boundary. This module is the
//! NAPI equivalent of the *responsibilities* those macros perform:
//!
//! * `#[get("/p")]` / `routes!` -> [`serve`] / [`Alloy::serve`] + the route-table parser, which at
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

/// Convenience alias for the threadsafe function each JS route handler is
/// wrapped in.
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

/// Wire type sent from Rust -> JS for every request, mirroring the request data
/// Rocket's generated route handler would have extracted before calling the user
/// fn.
#[napi]
pub struct RequestInput {
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) query: String,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body: Option<Vec<u8>>,
    pub(crate) params: HashMap<String, String>,
}

#[napi]
impl RequestInput {
    #[napi(getter)]
    pub fn method(&self) -> String {
        self.method.clone()
    }

    #[napi(getter)]
    pub fn path(&self) -> String {
        self.path.clone()
    }

    #[napi(getter)]
    pub fn query(&self) -> String {
        self.query.clone()
    }

    #[napi(getter)]
    pub fn headers(&self) -> HashMap<String, String> {
        self.headers.clone()
    }

    #[napi(getter)]
    pub fn body(&self) -> Option<Buffer> {
        self.body.as_ref().map(|b| Buffer::from(b.clone()))
    }

    #[napi(getter)]
    pub fn params(&self) -> HashMap<String, String> {
        self.params.clone()
    }

    #[napi]
    pub fn json(&self) -> napi::Result<serde_json::Value> {
        if let Some(b) = &self.body {
            serde_json::from_slice(b)
                .map_err(|e| napi::Error::new(Status::InvalidArg, format!("invalid JSON: {e}")))
        } else {
            Err(napi::Error::new(Status::InvalidArg, "no request body"))
        }
    }

    #[napi]
    pub fn text(&self) -> String {
        if let Some(b) = &self.body {
            String::from_utf8_lossy(b).to_string()
        } else {
            String::new()
        }
    }
}

/// Init passed to the `Response` constructors / helpers.
#[napi(object)]
pub struct ResponseInit {
    pub status: Option<u32>,
    pub headers: Option<HashMap<String, String>>,
}

/// The `Response` value a JS handler returns.
#[napi(object)]
pub struct Response {
    pub status: u32,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

/// `new Response(body, init)` (Bun/Web shape) — `body` may be a `string`.
#[napi]
pub fn response(body: Option<String>, init: Option<ResponseInit>) -> Response {
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

/// `Response.json(data, init?)` — stringifies `data` as JSON with the
/// `application/json` content type.
#[napi]
pub fn response_json(data: serde_json::Value, init: Option<ResponseInit>) -> napi::Result<Response> {
    let init = init.unwrap_or(ResponseInit {
        status: None,
        headers: None,
    });
    let bytes = serde_json::to_vec(&data)
        .map_err(|e| napi::Error::new(Status::GenericFailure, e.to_string()))?;
    let mut headers = init.headers.unwrap_or_default();
    headers.insert("content-type".to_string(), "application/json".to_string());
    Ok(Response {
        status: init.status.unwrap_or(200),
        headers,
        body: Some(bytes),
    })
}

/// `Response.redirect(url, status?)` — `status` defaults to 302.
#[napi]
pub fn response_redirect(url: String, status: Option<u32>) -> Response {
    let mut headers = HashMap::new();
    headers.insert("location".to_string(), url);
    Response {
        status: status.unwrap_or(302),
        headers,
        body: None,
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
    File {
        path: String,
    },
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

/// Extract the request data Rocket matched into the JS-facing [`RequestInput`],
/// including any `:param` / `<param>` path segments.
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

/// Align a Rocket route pattern's `<param>` segments with the actual request
/// path segments to populate `req.params`.
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

/// Translate a Bun/Express-style pattern (`:name`, `*`) into Rocket syntax
/// (`<name>`, `<_..>`).
fn translate_pattern(pattern: &str) -> String {
    let mut out = String::with_capacity(pattern.len());
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

/// Build a Rocket `Response` from a JS handler's `Response`.
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

/// Methods expanded when a route value is a bare function / static / file.
const ALL_METHODS: &[Method] = &[
    Method::Get,
    Method::Post,
    Method::Put,
    Method::Delete,
    Method::Patch,
    Method::Head,
    Method::Options,
];

/// Inspect one route value and append Rocket routes for it.
#[allow(deprecated)]
fn mount_value<'env>(
    value: Unknown<'env>,
    pattern: &str,
    methods: &[Method],
    out: &mut Vec<Route>,
) -> napi::Result<()> {
    match value.get_type()? {
        ValueType::Function => {
            let f = to_js_function(value);
            let tsfn: HandlerTsfn =
                f.create_threadsafe_function::<RequestInput, RequestInput, Promise<Response>, Status, _, false, false, 0>(
                    |ctx| Ok(ctx.value),
                )?;
            push_kind(RouteKind::Fn(Arc::new(tsfn)), pattern, methods, out);
        }
        ValueType::String => {
            let s: String = unsafe { value.cast()? };
            let mut headers = HashMap::new();
            headers.insert("content-type".to_string(), "text/plain; charset=utf-8".to_string());
            let resp = Response {
                status: 200,
                headers,
                body: Some(s.into_bytes()),
            };
            push_kind(RouteKind::Static(Arc::new(resp)), pattern, methods, out);
        }
        ValueType::Object => {
            let obj: Object = unsafe { value.clone().cast()? };
            if obj.has_named_property("path")? {
                let path: String = obj.get_named_property("path")?;
                push_kind(RouteKind::File { path }, pattern, methods, out);
            } else if obj.has_named_property("status")? || obj.has_named_property("headers")? || obj.has_named_property("body")? {
                let status: u32 = if obj.has_named_property("status")? {
                    obj.get_named_property::<Option<u32>>("status")?.unwrap_or(200)
                } else {
                    200
                };
                let headers: HashMap<String, String> = if obj.has_named_property("headers")? {
                    obj.get_named_property::<Option<HashMap<String, String>>>("headers")?.unwrap_or_default()
                } else {
                    HashMap::new()
                };
                let body: Option<Vec<u8>> = if obj.has_named_property("body")? {
                    let b: Option<Unknown<'env>> = obj.get_named_property("body")?;
                    match b {
                        Some(bv) => match bv.get_type()? {
                            ValueType::String => {
                                let s: String = unsafe { bv.cast()? };
                                Some(s.into_bytes())
                            }
                            ValueType::Object => {
                                let buf: Option<Buffer> = unsafe { bv.cast().ok() };
                                buf.map(|b| b.to_vec())
                            }
                            _ => None,
                        },
                        None => None,
                    }
                } else {
                    None
                };
                let resp = Response { status, headers, body };
                push_kind(RouteKind::Static(Arc::new(resp)), pattern, methods, out);
            } else {
                let names = obj.get_property_names()?;
                let len = names.get_array_length_unchecked()?;
                for i in 0..len {
                    let method_str: String = names.get_element(i)?;
                    let method_val: Unknown<'env> = obj.get_named_property(&method_str)?;
                    let m: Method = method_str.to_uppercase().parse().map_err(|_| {
                        napi::Error::new(Status::GenericFailure, format!("invalid method: {method_str}"))
                    })?;
                    mount_value(method_val, pattern, &[m], out)?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

/// Append one [`RouteKind`] for every method in `methods`.
fn push_kind(kind: RouteKind, pattern: &str, methods: &[Method], out: &mut Vec<Route>) {
    let handler = JsHandler(Arc::new(kind));
    let rocket_pattern = translate_pattern(pattern);
    for m in methods {
        out.push(Route::new(*m, &rocket_pattern, handler.clone()));
    }
}

/// Shared shutdown handle slot for an individual server instance.
type ShutdownSlot = Arc<Mutex<Option<Shutdown>>>;

/// Options for [`serve`] (scalar config only; `routes`/`fetch` are parameters or inside option object).
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

/// Helper to parse overloaded `serve()` arguments.
fn parse_serve_args<'env>(
    options_or_routes: Option<Unknown<'env>>,
    fetch_or_options: Option<Unknown<'env>>,
    maybe_options: Option<ServeOptions>,
) -> napi::Result<(
    Option<Unknown<'env>>,
    Option<Unknown<'env>>,
    ServeOptions,
)> {
    let mut routes: Option<Unknown<'env>> = None;
    let mut fetch: Option<Unknown<'env>> = None;
    let mut options = ServeOptions {
        port: None,
        hostname: None,
        unix: None,
    };

    if let Some(arg1) = options_or_routes {
        if arg1.get_type()? == ValueType::Object {
            let obj: Object = unsafe { arg1.clone().cast()? };
            let has_routes = obj.has_named_property("routes")?;
            let has_fetch = obj.has_named_property("fetch")?;
            let has_port = obj.has_named_property("port")?;
            let has_hostname = obj.has_named_property("hostname")?;
            let has_unix = obj.has_named_property("unix")?;

            if has_routes || has_fetch || has_port || has_hostname || has_unix {
                if has_routes {
                    routes = Some(obj.get_named_property("routes")?);
                }
                if has_fetch {
                    fetch = Some(obj.get_named_property("fetch")?);
                }
                if has_port {
                    options.port = obj.get_named_property("port")?;
                }
                if has_hostname {
                    options.hostname = obj.get_named_property("hostname")?;
                }
                if has_unix {
                    options.unix = obj.get_named_property("unix")?;
                }
            } else {
                routes = Some(arg1);
                if let Some(arg2) = fetch_or_options {
                    if arg2.get_type()? == ValueType::Function {
                        fetch = Some(arg2);
                    } else if arg2.get_type()? == ValueType::Object {
                        let opts: ServeOptions = unsafe { arg2.cast()? };
                        options = opts;
                    }
                }
                if let Some(opts) = maybe_options {
                    options = opts;
                }
            }
        }
    }

    Ok((routes, fetch, options))
}

/// NAPI implementation of `Alloy.serve({ routes, fetch, port, hostname, unix })` / `Bun.serve`.
#[napi]
pub fn serve(
    options_or_routes: Option<Unknown<'_>>,
    fetch_or_options: Option<Unknown<'_>>,
    maybe_options: Option<ServeOptions>,
) -> napi::Result<Server> {
    let (routes_val, fetch_val, options) = parse_serve_args(options_or_routes, fetch_or_options, maybe_options)?;

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

    let env_port = std::env::var("BUN_PORT")
        .or_else(|_| std::env::var("PORT"))
        .or_else(|_| std::env::var("NODE_PORT"))
        .ok()
        .and_then(|p| p.parse::<u16>().ok());

    let req_port = options.port.map(|p| p as u16).or(env_port).unwrap_or(3000);

    let (port, actual_address) = if req_port == 0 {
        let listener = std::net::TcpListener::bind((address, 0))
            .map_err(|e| napi::Error::new(Status::GenericFailure, format!("Failed to bind to port 0: {e}")))?;
        let bound_addr = listener.local_addr()
            .map_err(|e| napi::Error::new(Status::GenericFailure, e.to_string()))?;
        (bound_addr.port(), bound_addr.ip())
    } else {
        (req_port, address)
    };

    let display_host = if hostname == "0.0.0.0" {
        "localhost".to_string()
    } else {
        hostname.clone()
    };
    let url = format!("http://{}:{}", display_host, port);

    let mut out: Vec<Route> = Vec::new();
    if let Some(table) = routes_val {
        if table.get_type()? == ValueType::Object {
            let table: Object = unsafe { table.cast()? };
            let names = table.get_property_names()?;
            let len = names.get_array_length_unchecked()?;
            for i in 0..len {
                let pattern: String = names.get_element(i)?;
                let value: Unknown<'_> = table.get_named_property(&pattern)?;
                mount_value(value, &pattern, ALL_METHODS, &mut out)?;
            }
        }
    }

    if let Some(fetch) = fetch_val {
        let f = to_js_function(fetch);
        let tsfn: HandlerTsfn =
            f.create_threadsafe_function::<RequestInput, RequestInput, Promise<Response>, Status, _, false, false, 0>(
                |ctx| Ok(ctx.value),
            )?;
        let tsfn = Arc::new(tsfn);
        for m in ALL_METHODS {
            let mut route = Route::new(*m, "/<_..>", JsHandler(Arc::new(RouteKind::Fn(tsfn.clone()))));
            route.rank = 10;
            out.push(route);
        }
    }

    let mut config = Config::default();
    config.port = port;
    config.address = actual_address;

    let slot: ShutdownSlot = Arc::new(Mutex::new(None));
    let slot_clone = slot.clone();

    let rocket = rocket::build()
        .configure(config)
        .mount("/", out)
        .attach(AdHoc::on_liftoff("capture-shutdown", move |orbit| {
            Box::pin(async move {
                *slot_clone.lock().unwrap() = Some(orbit.shutdown());
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
        shutdown_slot: slot,
    })
}

/// `Alloy` namespace exposing `Alloy.serve`, `Alloy.file`, `Alloy.response`, `Alloy.responseJson`, `Alloy.responseRedirect`.
#[napi]
pub struct Alloy;

#[napi]
impl Alloy {
    #[napi]
    pub fn serve(
        options_or_routes: Option<Unknown<'_>>,
        fetch_or_options: Option<Unknown<'_>>,
        maybe_options: Option<ServeOptions>,
    ) -> napi::Result<Server> {
        serve(options_or_routes, fetch_or_options, maybe_options)
    }

    #[napi]
    pub fn file(path: String) -> StaticFile {
        file(path)
    }

    #[napi]
    pub fn response(body: Option<String>, init: Option<ResponseInit>) -> Response {
        response(body, init)
    }

    #[napi]
    pub fn response_json(data: serde_json::Value, init: Option<ResponseInit>) -> napi::Result<Response> {
        response_json(data, init)
    }

    #[napi]
    pub fn response_redirect(url: String, status: Option<u32>) -> Response {
        response_redirect(url, status)
    }
}

/// `Bun` namespace exposing `Bun.serve` and `Bun.file`.
#[napi]
pub struct Bun;

#[napi]
impl Bun {
    #[napi]
    pub fn file(path: String) -> StaticFile {
        file(path)
    }

    #[napi]
    pub fn serve(
        options_or_routes: Option<Unknown<'_>>,
        fetch_or_options: Option<Unknown<'_>>,
        maybe_options: Option<ServeOptions>,
    ) -> napi::Result<Server> {
        serve(options_or_routes, fetch_or_options, maybe_options)
    }
}

// ---------------------------------------------------------------------------
// Low-level registry API: `register_route`
// ---------------------------------------------------------------------------

static ROUTES: OnceLock<Mutex<Vec<Route>>> = OnceLock::new();

fn routes() -> &'static Mutex<Vec<Route>> {
    ROUTES.get_or_init(|| Mutex::new(Vec::new()))
}

/// Low level route registration.
#[napi]
pub fn register_route(method: String, path: String, handler: Unknown<'_>) -> napi::Result<()> {
    let f = to_js_function(handler);
    let tsfn: HandlerTsfn =
        f.create_threadsafe_function::<RequestInput, RequestInput, Promise<Response>, Status, _, false, false, 0>(
            |ctx| Ok(ctx.value),
        )?;
    let m: Method = method
        .parse()
        .map_err(|_| napi::Error::new(Status::GenericFailure, format!("invalid method: {method}")))?;
    let mut routes = routes().lock().unwrap();
    push_kind(RouteKind::Fn(Arc::new(tsfn)), &path, &[m], &mut routes);
    Ok(())
}

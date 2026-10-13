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
//!
//! The public surface below (`serve`, `Alloy`, `Bun`, `Response`, `file`,
//! `response`, `responseJson`, `responseRedirect`, `register_route`, `Server`)
//! is the single source of truth for Bun/Alloy-compatible HTTP servers powered
//! by Rocket.

use std::collections::HashMap;
use std::io::Cursor;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Arc, Mutex, OnceLock};

use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
#[allow(deprecated)]
use napi::{JsFunction, Unknown, ValueType};
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

/// Convert a `napi::Unknown` (received as a `#[napi]` parameter or parsed out of
/// a JS object) into a `napi::JsFunction` so we can call
/// `create_threadsafe_function`.
#[allow(deprecated)]
fn to_js_function(handler: Unknown) -> JsFunction {
    debug_assert_eq!(
        std::mem::size_of::<Unknown>(),
        std::mem::size_of::<JsFunction>()
    );
    unsafe { std::mem::transmute_copy::<Unknown, JsFunction>(&handler) }
}

/// Create a ThreadsafeFunction wrapping a JS route handler function.
#[allow(deprecated)]
fn create_handler_tsfn(handler: Unknown) -> napi::Result<HandlerTsfn> {
    let f = to_js_function(handler);
    f.create_threadsafe_function::<RequestInput, RequestInput, Promise<Response>, Status, _, false, false, 0>(
        |ctx| Ok(ctx.value),
    )
}

/// Wire type sent from Rust -> JS for every request, mirroring the request data
/// Rocket's generated route handler would have extracted before calling the user
/// fn. `params` carries any `:name` / `<name>` path segments.
#[napi(object)]
#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
pub struct ResponseInit {
    pub status: Option<u32>,
    pub headers: Option<HashMap<String, String>>,
}

/// The `Response` value a JS handler returns or routes serve directly.
#[napi(object)]
#[derive(Clone, Debug)]
pub struct Response {
    pub status: u32,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

/// Free function helper `response(body, init)`.
#[napi]
pub fn response(body: Option<Unknown<'_>>, init: Option<ResponseInit>) -> napi::Result<Response> {
    let init = init.unwrap_or(ResponseInit {
        status: None,
        headers: None,
    });
    let body_bytes = if let Some(b) = body {
        match b.get_type()? {
            ValueType::String => {
                let s: String = unsafe { b.cast()? };
                Some(s.into_bytes())
            }
            ValueType::Object => {
                if let Ok(buf) = unsafe { b.clone().cast::<Buffer>() } {
                    Some(buf.to_vec())
                } else if let Ok(s) = unsafe { b.cast::<String>() } {
                    Some(s.into_bytes())
                } else {
                    None
                }
            }
            _ => None,
        }
    } else {
        None
    };
    Ok(Response {
        status: init.status.unwrap_or(200),
        headers: init.headers.unwrap_or_default(),
        body: body_bytes,
    })
}

/// Free function helper `responseJson(data, init)`.
#[napi]
pub fn response_json(data: serde_json::Value, init: Option<ResponseInit>) -> napi::Result<Response> {
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

/// Lazily-served static file marker returned by [`file`].
#[napi(object)]
#[derive(Clone, Debug)]
pub struct StaticFile {
    pub path: String,
}

/// `file(path)` — marks a route value as a file served from disk.
#[napi]
pub fn file(path: String) -> StaticFile {
    StaticFile { path }
}

/// Struct exposing Bun / Alloy runtime compatibility namespace `Alloy`.
#[napi]
pub struct Alloy;

#[napi]
impl Alloy {
    /// `Alloy.serve(options)` — Bun.serve-compatible server bootstrapper.
    #[napi]
    pub fn serve(
        options_or_routes: Option<Unknown<'_>>,
        fetch: Option<Unknown<'_>>,
        options: Option<ServeOptions>,
    ) -> napi::Result<Server> {
        serve(options_or_routes, fetch, options)
    }

    /// `Alloy.file(path)` — static file marker helper.
    #[napi]
    pub fn file(path: String) -> StaticFile {
        file(path)
    }
}

/// Struct exposing Bun runtime compatibility namespace `Bun`.
#[napi]
pub struct Bun;

#[napi]
impl Bun {
    /// `Bun.serve(options)` — Bun.serve-compatible server bootstrapper.
    #[napi]
    pub fn serve(
        options_or_routes: Option<Unknown<'_>>,
        fetch: Option<Unknown<'_>>,
        options: Option<ServeOptions>,
    ) -> napi::Result<Server> {
        serve(options_or_routes, fetch, options)
    }

    /// `Bun.file(path)` — static file marker helper.
    #[napi]
    pub fn file(path: String) -> StaticFile {
        file(path)
    }
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
            let tsfn = create_handler_tsfn(value)?;
            push_kind(RouteKind::Fn(Arc::new(tsfn)), pattern, methods, out);
        }
        ValueType::Object => {
            let v = value.clone();
            if let Ok(fr) = unsafe { v.clone().cast::<StaticFile>() } {
                push_kind(
                    RouteKind::File { path: fr.path },
                    pattern,
                    methods,
                    out,
                );
            } else if let Ok(resp) = unsafe { v.clone().cast::<Response>() } {
                push_kind(RouteKind::Static(Arc::new(resp)), pattern, methods, out);
            } else if let Ok(obj) = unsafe { v.clone().cast::<Object>() } {
                if obj.has_named_property("status")? {
                    let status: u32 = obj.get_named_property("status").unwrap_or(200);
                    let headers: HashMap<String, String> =
                        obj.get_named_property("headers").unwrap_or_default();
                    let body: Option<Vec<u8>> = obj.get_named_property("body").ok();
                    let resp = Response { status, headers, body };
                    push_kind(RouteKind::Static(Arc::new(resp)), pattern, methods, out);
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
                        mount_value(method_value, pattern, &[m], out)?;
                    }
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

/// Shared shutdown handle.
type ShutdownSlot = Arc<Mutex<Option<Shutdown>>>;
static SHUTDOWN: OnceLock<ShutdownSlot> = OnceLock::new();

/// NAPI equivalent of `Alloy.serve(options)` or `serve(routes, fetch, options)`.
#[napi]
pub fn serve(
    options_or_routes: Option<Unknown<'_>>,
    fetch: Option<Unknown<'_>>,
    options: Option<ServeOptions>,
) -> napi::Result<Server> {
    let mut routes_val: Option<Unknown<'_>> = None;
    let mut fetch_val: Option<Unknown<'_>> = fetch;
    let mut port_val: Option<u32> = options.as_ref().and_then(|o| o.port);
    let mut hostname_val: Option<String> = options.as_ref().and_then(|o| o.hostname.clone());
    let mut unix_val: Option<String> = options.as_ref().and_then(|o| o.unix.clone());

    if let Some(arg0) = options_or_routes {
        if arg0.get_type()? == ValueType::Object {
            let obj: Object = unsafe { arg0.cast()? };
            let has_routes = obj.has_named_property("routes")?;
            let has_fetch = obj.has_named_property("fetch")?;
            let has_port = obj.has_named_property("port")?;
            let has_hostname = obj.has_named_property("hostname")?;
            let has_unix = obj.has_named_property("unix")?;

            if has_routes || has_fetch || has_port || has_hostname || has_unix {
                if has_routes {
                    routes_val = Some(obj.get_named_property("routes")?);
                }
                if has_fetch {
                    fetch_val = Some(obj.get_named_property("fetch")?);
                }
                if has_port {
                    let p: Option<u32> = obj.get_named_property("port")?;
                    if p.is_some() {
                        port_val = p;
                    }
                }
                if has_hostname {
                    let h: Option<String> = obj.get_named_property("hostname")?;
                    if h.is_some() {
                        hostname_val = h;
                    }
                }
                if has_unix {
                    let u: Option<String> = obj.get_named_property("unix")?;
                    if u.is_some() {
                        unix_val = u;
                    }
                }
            } else {
                routes_val = Some(arg0);
            }
        } else {
            routes_val = Some(arg0);
        }
    }

    let port = if let Some(p) = port_val {
        if p == 0 {
            let hostname_str = hostname_val.as_deref().unwrap_or("0.0.0.0");
            let addr = if hostname_str == "localhost" {
                "127.0.0.1"
            } else {
                hostname_str
            };
            let listener = std::net::TcpListener::bind(format!("{}:0", addr))
                .or_else(|_| std::net::TcpListener::bind("127.0.0.1:0"))
                .map_err(|e| napi::Error::new(Status::GenericFailure, format!("Failed to bind port 0: {e}")))?;
            let bound_port = listener
                .local_addr()
                .map_err(|e| napi::Error::new(Status::GenericFailure, format!("Failed to get local port: {e}")))?
                .port();
            drop(listener);
            bound_port
        } else {
            p as u16
        }
    } else {
        std::env::var("BUN_PORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok())
            .or_else(|| std::env::var("PORT").ok().and_then(|v| v.parse::<u16>().ok()))
            .or_else(|| std::env::var("NODE_PORT").ok().and_then(|v| v.parse::<u16>().ok()))
            .unwrap_or(3000)
    };

    let hostname = hostname_val.unwrap_or_else(|| "0.0.0.0".to_string());
    let address: IpAddr = match hostname.as_str() {
        "localhost" => IpAddr::V4(Ipv4Addr::LOCALHOST),
        other => other
            .parse()
            .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
    };
    let display_host = if hostname == "0.0.0.0" { "localhost" } else { &hostname };
    let url = format!("http://{}:{}", display_host, port);

    let mut out: Vec<Route> = Vec::new();
    if let Some(table) = routes_val {
        let table: Object = unsafe { table.cast()? };
        let names = table.get_property_names()?;
        let len = names.get_array_length_unchecked()?;
        for i in 0..len {
            let pattern: String = names.get_element(i)?;
            let value: Unknown<'_> = table.get_named_property(&pattern)?;
            mount_value(value, &pattern, ALL_METHODS, &mut out)?;
        }
    }

    if let Some(fetch) = fetch_val {
        let tsfn = create_handler_tsfn(fetch)?;
        let tsfn = Arc::new(tsfn);
        for m in ALL_METHODS {
            push_kind(RouteKind::Fn(tsfn.clone()), "/<_..>", &[*m], &mut out);
        }
    }

    let mut config = Config::default();
    config.port = port;
    config.address = address;

    let slot: ShutdownSlot = Arc::new(Mutex::new(None));
    SHUTDOWN.get_or_init(|| slot.clone());

    let rocket = rocket::build()
        .configure(config)
        .mount("/", out)
        .attach(AdHoc::on_liftoff("capture-shutdown", move |orbit| {
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
        unix: unix_val,
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
    pub unix: Option<String>,
}

#[napi]
impl Server {
    /// Gracefully stop the server.
    #[napi]
    pub fn stop(&self) -> napi::Result<()> {
        if let Some(slot) = SHUTDOWN.get() {
            if let Some(shutdown) = slot.lock().unwrap().take() {
                shutdown.notify();
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Low-level registry API: `register_route`
// ---------------------------------------------------------------------------

static ROUTES: OnceLock<Mutex<Vec<Route>>> = OnceLock::new();

fn routes() -> &'static Mutex<Vec<Route>> {
    ROUTES.get_or_init(|| Mutex::new(Vec::new()))
}

/// NAPI equivalent of `#[get("/p")] fn ..`
#[napi]
pub fn register_route(method: String, path: String, handler: Unknown<'_>) -> napi::Result<()> {
    let tsfn = create_handler_tsfn(handler)?;
    let m: Method = method
        .parse()
        .map_err(|_| napi::Error::new(Status::GenericFailure, format!("invalid method: {method}")))?;
    let mut routes = routes().lock().unwrap();
    push_kind(RouteKind::Fn(Arc::new(tsfn)), &path, &[m], &mut routes);
    Ok(())
}

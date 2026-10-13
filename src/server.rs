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
use napi::{JsFunction, JsObject, JsString, Unknown, ValueType};
use napi_derive::napi;

use rocket::config::Config;
use rocket::data::ByteUnit;
use rocket::fairing::AdHoc;
use rocket::http::{Header as RocketHeader, Method, Status as RocketStatus};
use rocket::request::Request;
use rocket::response::Response as RocketResponse;
use rocket::route::{Handler, Outcome as RouteOutcome, Route};
use rocket::{Data, Shutdown};

type HandlerTsfn =
    ThreadsafeFunction<RequestInput, Promise<ResponseValue>, RequestInput, Status, false, false, 0>;

#[allow(deprecated)]
fn to_js_function(handler: Unknown<'_>) -> JsFunction {
    debug_assert_eq!(
        std::mem::size_of::<Unknown<'_>>(),
        std::mem::size_of::<JsFunction>()
    );
    unsafe { std::mem::transmute_copy::<Unknown<'_>, JsFunction>(&handler) }
}

/// Wire type sent from Rust -> JS for every request, mirroring the request data
/// Rocket's generated route handler would have extracted before calling the user
/// fn. `params` carries any `:name` / `<name>` path segments.
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

/// The response payload extracted from JS.
#[napi(object)]
#[derive(Clone, Debug)]
pub struct ResponseValue {
    pub status: u32,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

/// The `Response` class exposed to JS: `new Response(body, init)`.
#[napi]
pub struct Response {
    status_val: u32,
    headers_val: HashMap<String, String>,
    body_val: Option<Vec<u8>>,
}

#[napi]
impl Response {
    #[napi(constructor)]
    pub fn new(body: Option<Unknown<'_>>, init: Option<ResponseInit>) -> napi::Result<Self> {
        let init = init.unwrap_or(ResponseInit {
            status: None,
            headers: None,
        });
        let status_val = init.status.unwrap_or(200);
        let headers_val = init.headers.unwrap_or_default();
        let bytes = if let Some(b) = body {
            if let Ok(s) = unsafe { b.clone().cast::<JsString<'_>>() } {
                Some(s.into_utf8()?.as_str()?.as_bytes().to_vec())
            } else if let Ok(buf) = unsafe { b.clone().cast::<Buffer>() } {
                Some(buf.to_vec())
            } else {
                None
            }
        } else {
            None
        };
        Ok(Self {
            status_val,
            headers_val,
            body_val: bytes,
        })
    }

    #[napi]
    pub fn json(data: serde_json::Value, init: Option<ResponseInit>) -> napi::Result<ResponseValue> {
        let init = init.unwrap_or(ResponseInit {
            status: None,
            headers: None,
        });
        let bytes = serde_json::to_vec(&data)
            .map_err(|e| napi::Error::new(Status::GenericFailure, e.to_string()))?;
        let mut headers = init.headers.unwrap_or_default();
        headers.insert("content-type".to_string(), "application/json".to_string());
        Ok(ResponseValue {
            status: init.status.unwrap_or(200),
            headers,
            body: Some(bytes),
        })
    }

    #[napi]
    pub fn redirect(url: String, status: Option<u32>) -> ResponseValue {
        let mut headers = HashMap::new();
        headers.insert("location".to_string(), url);
        ResponseValue {
            status: status.unwrap_or(302),
            headers,
            body: None,
        }
    }

    #[napi(getter)]
    pub fn status(&self) -> u32 {
        self.status_val
    }

    #[napi(getter)]
    pub fn headers(&self) -> HashMap<String, String> {
        self.headers_val.clone()
    }

    #[napi(getter)]
    pub fn body(&self) -> Option<Vec<u8>> {
        self.body_val.clone()
    }
}

/// Free function helper: `response(body, init?)`.
#[napi]
pub fn response(body: Option<String>, init: Option<ResponseInit>) -> ResponseValue {
    let init = init.unwrap_or(ResponseInit {
        status: None,
        headers: None,
    });
    ResponseValue {
        status: init.status.unwrap_or(200),
        headers: init.headers.unwrap_or_default(),
        body: body.map(|s| s.into_bytes()),
    }
}

/// Free function helper: `responseJson(data, init?)`.
#[napi]
pub fn response_json(data: serde_json::Value, init: Option<ResponseInit>) -> napi::Result<ResponseValue> {
    Response::json(data, init)
}

/// Free function helper: `responseRedirect(url, status?)`.
#[napi]
pub fn response_redirect(url: String, status: Option<u32>) -> ResponseValue {
    Response::redirect(url, status)
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

/// Bun namespace providing Bun.file compatibility.
#[napi]
pub struct Bun;

#[napi]
impl Bun {
    #[napi]
    pub fn file(path: String) -> StaticFile {
        file(path)
    }
}

/// The resolved intent of one route entry after parsing the JS `routes` table.
#[derive(Clone)]
enum RouteKind {
    Fn(Arc<HandlerTsfn>),
    Static(Arc<ResponseValue>),
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

    let body = match data.open(ByteUnit::Megabyte(10)).into_bytes().await {
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
        } else if let Some(name) = seg.strip_prefix(':') {
            if let Some(v) = a_segs.get(idx) {
                params.insert(name.to_string(), (*v).to_string());
            }
        }
    }
    params
}

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

fn build_response(payload: &ResponseValue) -> RocketResponse<'static> {
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

#[allow(deprecated)]
fn create_handler_tsfn(f: JsFunction) -> napi::Result<HandlerTsfn> {
    f.create_threadsafe_function::<RequestInput, RequestInput, Promise<ResponseValue>, Status, _, false, false, 0>(
        |ctx| Ok(ctx.value),
    )
}

#[allow(deprecated)]
fn extract_response_value(val: Unknown<'_>) -> Option<ResponseValue> {
    if let Ok(obj) = unsafe { val.cast::<JsObject>() } {
        let status: u32 = obj.get_named_property("status").unwrap_or(200);
        let mut headers = HashMap::new();
        if let Ok(h_obj) = obj.get_named_property::<JsObject>("headers") {
            if let Ok(names) = h_obj.get_property_names() {
                if let Ok(len) = names.get_array_length_unchecked() {
                    for i in 0..len {
                        if let (Ok(name), Ok(v)) = (names.get_element::<String>(i), h_obj.get_named_property::<String>(&names.get_element::<String>(i).unwrap_or_default())) {
                            headers.insert(name, v);
                        }
                    }
                }
            }
        }
        let body = if let Ok(b_val) = obj.get_named_property::<Unknown<'_>>("body") {
            if let Ok(s) = unsafe { b_val.clone().cast::<JsString<'_>>() } {
                Some(s.into_utf8().ok()?.as_str().ok()?.as_bytes().to_vec())
            } else if let Ok(buf) = unsafe { b_val.clone().cast::<Buffer>() } {
                Some(buf.to_vec())
            } else {
                None
            }
        } else {
            None
        };
        Some(ResponseValue { status, headers, body })
    } else {
        None
    }
}

#[allow(deprecated)]
fn mount_value<'env>(
    value: Unknown<'env>,
    pattern: &str,
    methods: &[Method],
    rank: Option<isize>,
    out: &mut Vec<Route>,
) -> napi::Result<()> {
    match value.get_type()? {
        ValueType::Function => {
            let f = to_js_function(value);
            let tsfn = create_handler_tsfn(f)?;
            push_kind(RouteKind::Fn(Arc::new(tsfn)), pattern, methods, rank, out);
        }
        ValueType::Object => {
            let v = value.clone();
            let obj: JsObject = unsafe { v.clone().cast()? };
            if obj.has_named_property("path")? {
                let path: String = obj.get_named_property("path")?;
                push_kind(RouteKind::File { path }, pattern, methods, rank, out);
            } else if obj.has_named_property("status")? || obj.has_named_property("headers")? || obj.has_named_property("body")? {
                if let Some(resp) = extract_response_value(v) {
                    push_kind(RouteKind::Static(Arc::new(resp)), pattern, methods, rank, out);
                }
            } else {
                let names = obj.get_property_names()?;
                let len = names.get_array_length_unchecked()?;
                for i in 0..len {
                    let m_str: String = names.get_element(i)?;
                    if let Ok(m) = m_str.parse::<Method>() {
                        let method_val: Unknown<'env> = obj.get_named_property(&m_str)?;
                        mount_value(method_val, pattern, &[m], rank, out)?;
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn push_kind(
    kind: RouteKind,
    pattern: &str,
    methods: &[Method],
    rank: Option<isize>,
    out: &mut Vec<Route>,
) {
    let handler = JsHandler(Arc::new(kind));
    let rocket_pattern = translate_pattern(pattern);
    for m in methods {
        let mut route = Route::new(*m, &rocket_pattern, handler.clone());
        if let Some(r) = rank {
            route.rank = r;
        }
        out.push(route);
    }
}

type ShutdownSlot = Arc<Mutex<Option<Shutdown>>>;
static SHUTDOWN: OnceLock<ShutdownSlot> = OnceLock::new();

#[napi(object)]
pub struct ServeOptions {
    pub port: Option<u32>,
    pub hostname: Option<String>,
    pub unix: Option<String>,
}

fn resolve_port(options_port: Option<u32>) -> u16 {
    if let Some(p) = options_port {
        p as u16
    } else if let Ok(val) = std::env::var("BUN_PORT") {
        val.parse().unwrap_or(3000)
    } else if let Ok(val) = std::env::var("PORT") {
        val.parse().unwrap_or(3000)
    } else if let Ok(val) = std::env::var("NODE_PORT") {
        val.parse().unwrap_or(3000)
    } else {
        3000
    }
}

#[napi]
#[allow(deprecated)]
pub fn serve(
    routes_or_options: Option<Unknown<'_>>,
    fetch_or_options: Option<Unknown<'_>>,
    options: Option<ServeOptions>,
) -> napi::Result<Server> {
    let mut routes_val: Option<Unknown<'_>> = None;
    let mut fetch_val: Option<Unknown<'_>> = None;
    let mut opt_port: Option<u32> = options.as_ref().and_then(|o| o.port);
    let mut opt_hostname: Option<String> = options.as_ref().and_then(|o| o.hostname.clone());
    let mut opt_unix: Option<String> = options.as_ref().and_then(|o| o.unix.clone());

    if let Some(ref arg0) = routes_or_options {
        if arg0.get_type()? == ValueType::Object {
            let obj: JsObject = unsafe { arg0.clone().cast()? };
            let has_routes = obj.has_named_property("routes")?;
            let has_fetch = obj.has_named_property("fetch")?;
            let has_port = obj.has_named_property("port")?;
            let has_hostname = obj.has_named_property("hostname")?;
            let has_unix = obj.has_named_property("unix")?;

            if has_routes || has_fetch || has_port || has_hostname || has_unix {
                if has_routes {
                    routes_val = obj.get_named_property("routes").ok();
                }
                if has_fetch {
                    fetch_val = obj.get_named_property("fetch").ok();
                }
                if has_port {
                    opt_port = obj.get_named_property("port").ok();
                }
                if has_hostname {
                    opt_hostname = obj.get_named_property("hostname").ok();
                }
                if has_unix {
                    opt_unix = obj.get_named_property("unix").ok();
                }
            } else {
                routes_val = Some(arg0.clone());
                fetch_val = fetch_or_options.clone();
            }
        }
    }

    let port = resolve_port(opt_port);
    let hostname = opt_hostname.unwrap_or_else(|| "0.0.0.0".to_string());
    let address: IpAddr = match hostname.as_str() {
        "localhost" => IpAddr::V4(Ipv4Addr::LOCALHOST),
        other => other.parse().unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
    };

    let mut out: Vec<Route> = Vec::new();
    if let Some(table) = routes_val {
        if table.get_type()? == ValueType::Object {
            let table_obj: JsObject = unsafe { table.cast()? };
            let names = table_obj.get_property_names()?;
            let len = names.get_array_length_unchecked()?;
            for i in 0..len {
                let pattern: String = names.get_element(i)?;
                let value: Unknown<'_> = table_obj.get_named_property(&pattern)?;
                let rank = if pattern.contains('*') { Some(5) } else { None };
                mount_value(value, &pattern, ALL_METHODS, rank, &mut out)?;
            }
        }
    }

    if let Some(fetch_fn) = fetch_val {
        if fetch_fn.get_type()? == ValueType::Function {
            let f = to_js_function(fetch_fn);
            let tsfn = create_handler_tsfn(f)?;
            push_kind(RouteKind::Fn(Arc::new(tsfn)), "*", ALL_METHODS, Some(10), &mut out);
        }
    }

    let mut config = Config::default();
    config.port = port;
    config.address = address;

    let slot: ShutdownSlot = Arc::new(Mutex::new(None));
    SHUTDOWN.get_or_init(|| slot.clone());

    let (port_tx, port_rx) = std::sync::mpsc::channel::<(u16, String)>();

    let rocket = rocket::build()
        .configure(config)
        .mount("/", out)
        .attach(AdHoc::on_liftoff("capture-shutdown-and-port", move |orbit| {
            let bound_port = orbit.config().port;
            let bound_addr = orbit.config().address.to_string();
            let _ = port_tx.send((bound_port, bound_addr));
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

    let (actual_port, actual_host) = port_rx
        .recv_timeout(std::time::Duration::from_secs(3))
        .unwrap_or((port, hostname.clone()));

    let url = if let Some(ref unix_path) = opt_unix {
        format!("http://unix:{unix_path}")
    } else if actual_host == "0.0.0.0" || actual_host == "127.0.0.1" {
        format!("http://localhost:{actual_port}")
    } else {
        format!("http://{actual_host}:{actual_port}")
    };

    Ok(Server {
        url,
        port: actual_port as u32,
    })
}

/// NAPI struct exposing `Alloy.serve`.
#[napi]
pub struct Alloy;

#[napi]
impl Alloy {
    #[napi]
    pub fn serve(
        routes_or_options: Option<Unknown<'_>>,
        fetch_or_options: Option<Unknown<'_>>,
        options: Option<ServeOptions>,
    ) -> napi::Result<Server> {
        serve(routes_or_options, fetch_or_options, options)
    }
}

/// Handle returned by [`serve`].
#[napi]
pub struct Server {
    pub url: String,
    pub port: u32,
}

#[napi]
impl Server {
    #[napi]
    pub fn stop(&self) -> napi::Result<()> {
        if let Some(slot) = SHUTDOWN.get() {
            if let Some(shutdown) = slot.lock().unwrap().take() {
                shutdown.notify();
            }
        }
        Ok(())
    }

    #[napi(getter)]
    pub fn url(&self) -> String {
        self.url.clone()
    }

    #[napi(getter)]
    pub fn port(&self) -> u32 {
        self.port
    }
}

static ROUTES: OnceLock<Mutex<Vec<Route>>> = OnceLock::new();

fn routes() -> &'static Mutex<Vec<Route>> {
    ROUTES.get_or_init(|| Mutex::new(Vec::new()))
}

#[napi]
pub fn register_route(method: String, path: String, handler: Unknown<'_>) -> napi::Result<()> {
    let f = to_js_function(handler);
    let tsfn = create_handler_tsfn(f)?;
    let m: Method = method
        .parse()
        .map_err(|_| napi::Error::new(Status::GenericFailure, format!("invalid method: {method}")))?;
    let mut routes = routes().lock().unwrap();
    push_kind(RouteKind::Fn(Arc::new(tsfn)), &path, &[m], None, &mut routes);
    Ok(())
}

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

type HandlerTsfn =
    ThreadsafeFunction<RequestInput, Promise<Response>, (RequestInput,), Status, true, false, 0>;

#[allow(deprecated)]
fn to_js_function(handler: Unknown) -> napi::Result<JsFunction> {
    if handler.get_type()? != ValueType::Function {
        return Err(napi::Error::new(
            Status::InvalidArg,
            "Expected a JS function handler",
        ));
    }
    let f: JsFunction = unsafe { std::mem::transmute_copy::<Unknown, JsFunction>(&handler) };
    Ok(f)
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

/// The `Response` value a JS handler returns.
#[derive(Clone)]
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
    headers.insert("Content-Type".to_string(), "application/json".to_string());
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
    headers.insert("Location".to_string(), url);
    Response {
        status: status.unwrap_or(302),
        headers,
        body: None,
    }
}

/// Lazily-served static file marker returned by [`file`].
#[derive(Clone)]
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
                let output = match tsfn.call_async(Ok(input)).await {
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
    let clean_pattern = pattern.split('?').next().unwrap_or(pattern);
    let clean_path = path.split('?').next().unwrap_or(path);

    let p_segs: Vec<&str> = clean_pattern
        .trim_start_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();
    let a_segs: Vec<&str> = clean_path
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

fn mount_value<'env>(
    value: Unknown<'env>,
    pattern: &str,
    methods: &[Method],
    out: &mut Vec<Route>,
) -> napi::Result<()> {
    match value.get_type()? {
        ValueType::Function => {
            let f = to_js_function(value)?;
            let tsfn: HandlerTsfn =
                f.create_threadsafe_function::<RequestInput, (RequestInput,), Promise<Response>, Status, _, true, false, 0>(
                    |ctx| Ok((ctx.value,)),
                )?;
            push_kind(RouteKind::Fn(Arc::new(tsfn)), pattern, methods, out);
        }
        ValueType::Object => {
            let obj = value.coerce_to_object()?;
            if obj.has_named_property("path")? && !obj.has_named_property("status")? {
                if let Ok(path) = obj.get_named_property::<String>("path") {
                    push_kind(
                        RouteKind::File { path },
                        pattern,
                        methods,
                        out,
                    );
                    return Ok(());
                }
            }
            if obj.has_named_property("status")? {
                let status = obj.get_named_property::<u32>("status")?;
                let headers = obj.get_named_property::<HashMap<String, String>>("headers").unwrap_or_default();
                let body = obj.get_named_property::<Option<Vec<u8>>>("body").unwrap_or(None);
                push_kind(
                    RouteKind::Static(Arc::new(Response { status, headers, body })),
                    pattern,
                    methods,
                    out,
                );
                return Ok(());
            }
            let names = obj.get_property_names()?;
            let len = names.get_array_length_unchecked()?;
            for i in 0..len {
                let method: String = names.get_element(i)?;
                let m_upper = method.to_ascii_uppercase();
                if let Ok(m) = m_upper.parse::<Method>() {
                    let method_value: Unknown<'_> = obj.get_named_property(&method)?;
                    mount_value(method_value, pattern, &[m], out)?;
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
    for m in methods {
        out.push(Route::new(*m, &rocket_pattern, handler.clone()));
    }
}

type ShutdownSlot = Arc<Mutex<Option<Shutdown>>>;

#[napi]
pub fn serve(
    routes_or_options: Option<Unknown<'_>>,
    fetch: Option<Unknown<'_>>,
    options: Option<ServeOptions>,
) -> napi::Result<Server> {
    let mut real_routes: Option<Unknown<'_>> = None;
    let mut real_fetch: Option<Unknown<'_>> = fetch;
    let mut opts = options.unwrap_or_else(|| ServeOptions {
        port: None,
        hostname: None,
        unix: None,
    });

    if let Some(arg) = routes_or_options {
        if arg.get_type()? == ValueType::Object {
            let obj = arg.coerce_to_object()?;
            let has_routes = obj.has_named_property("routes")?;
            let has_fetch = obj.has_named_property("fetch")?;
            let has_port = obj.has_named_property("port")?;
            let has_hostname = obj.has_named_property("hostname")?;

            if has_routes || has_fetch || has_port || has_hostname {
                if has_routes {
                    real_routes = Some(obj.get_named_property("routes")?);
                }
                if has_fetch {
                    real_fetch = Some(obj.get_named_property("fetch")?);
                }
                if has_port {
                    if let Ok(p) = obj.get_named_property::<u32>("port") {
                        opts.port = Some(p);
                    }
                }
                if has_hostname {
                    if let Ok(h) = obj.get_named_property::<String>("hostname") {
                        opts.hostname = Some(h);
                    }
                }
            } else {
                real_routes = Some(arg);
            }
        } else {
            real_routes = Some(arg);
        }
    }

    let mut port = opts.port.unwrap_or(3000) as u16;
    let hostname = opts
        .hostname
        .clone()
        .unwrap_or_else(|| "0.0.0.0".to_string());
    let address: IpAddr = match hostname.as_str() {
        "localhost" => IpAddr::V4(Ipv4Addr::LOCALHOST),
        other => other
            .parse()
            .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
    };

    if port == 0 {
        if let Ok(listener) = std::net::TcpListener::bind((address, 0)) {
            if let Ok(addr) = listener.local_addr() {
                port = addr.port();
            }
        }
    }

    let url = format!("http://{}:{}", hostname, port);

    let mut out: Vec<Route> = Vec::new();
    if let Some(table) = real_routes {
        let table = table.coerce_to_object()?;
        let names = table.get_property_names()?;
        let len = names.get_array_length_unchecked()?;
        for i in 0..len {
            let pattern: String = names.get_element(i)?;
            let value: Unknown<'_> = table.get_named_property(&pattern)?;
            mount_value(value, &pattern, ALL_METHODS, &mut out)?;
        }
    }

    if let Some(fetch_fn) = real_fetch {
        let f = to_js_function(fetch_fn)?;
        let tsfn: HandlerTsfn =
            f.create_threadsafe_function::<RequestInput, (RequestInput,), Promise<Response>, Status, _, true, false, 0>(
                |ctx| Ok((ctx.value,)),
            )?;
        let tsfn = Arc::new(tsfn);
        for m in ALL_METHODS {
            push_kind(RouteKind::Fn(tsfn.clone()), "<_..>", &[*m], &mut out);
        }
    }

    let mut config = Config::default();
    config.port = port;
    config.address = address;

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
        shutdown: slot,
    })
}

/// Options for [`serve`].
#[derive(Clone)]
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
    shutdown: ShutdownSlot,
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

/// Fluent router matching Rocket's `rocket::build()` and `#[get]` attributes.
#[napi]
pub struct App {
    routes: HashMap<String, HashMap<String, Arc<RouteKind>>>,
    options: ServeOptions,
}

#[napi]
impl App {
    #[napi(constructor)]
    pub fn new(options: Option<ServeOptions>) -> Self {
        Self {
            routes: HashMap::new(),
            options: options.unwrap_or_else(|| ServeOptions {
                port: None,
                hostname: None,
                unix: None,
            }),
        }
    }

    #[napi]
    pub fn get(&mut self, path: String, handler: Unknown<'_>) -> napi::Result<()> {
        self.register_method(path, "GET", handler)
    }

    #[napi]
    pub fn post(&mut self, path: String, handler: Unknown<'_>) -> napi::Result<()> {
        self.register_method(path, "POST", handler)
    }

    #[napi]
    pub fn put(&mut self, path: String, handler: Unknown<'_>) -> napi::Result<()> {
        self.register_method(path, "PUT", handler)
    }

    #[napi]
    pub fn delete(&mut self, path: String, handler: Unknown<'_>) -> napi::Result<()> {
        self.register_method(path, "DELETE", handler)
    }

    #[napi]
    pub fn patch(&mut self, path: String, handler: Unknown<'_>) -> napi::Result<()> {
        self.register_method(path, "PATCH", handler)
    }

    fn register_method(&mut self, path: String, method: &str, handler: Unknown<'_>) -> napi::Result<()> {
        let f = to_js_function(handler)?;
        let tsfn: HandlerTsfn =
            f.create_threadsafe_function::<RequestInput, (RequestInput,), Promise<Response>, Status, _, true, false, 0>(
                |ctx| Ok((ctx.value,)),
            )?;
        let kind = Arc::new(RouteKind::Fn(Arc::new(tsfn)));
        let entry = self.routes.entry(path).or_default();
        entry.insert(method.to_string(), kind);
        Ok(())
    }

    #[napi]
    pub fn launch(&mut self) -> napi::Result<Server> {
        let mut out: Vec<Route> = Vec::new();
        for (pattern, method_map) in &self.routes {
            for (method_str, kind) in method_map {
                if let Ok(m) = method_str.parse::<Method>() {
                    push_kind((**kind).clone(), pattern, &[m], &mut out);
                }
            }
        }

        let mut port = self.options.port.unwrap_or(3000) as u16;
        let hostname = self
            .options
            .hostname
            .clone()
            .unwrap_or_else(|| "0.0.0.0".to_string());
        let address: IpAddr = match hostname.as_str() {
            "localhost" => IpAddr::V4(Ipv4Addr::LOCALHOST),
            other => other
                .parse()
                .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
        };

        if port == 0 {
            if let Ok(listener) = std::net::TcpListener::bind((address, 0)) {
                if let Ok(addr) = listener.local_addr() {
                    port = addr.port();
                }
            }
        }

        let url = format!("http://{}:{}", hostname, port);

        let mut config = Config::default();
        config.port = port;
        config.address = address;

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
            shutdown: slot,
        })
    }
}

static ROUTES: OnceLock<Mutex<Vec<Route>>> = OnceLock::new();

fn routes() -> &'static Mutex<Vec<Route>> {
    ROUTES.get_or_init(|| Mutex::new(Vec::new()))
}

#[napi]
pub fn register_route(method: String, path: String, handler: Unknown<'_>) -> napi::Result<()> {
    let f = to_js_function(handler)?;
    let tsfn: HandlerTsfn =
        f.create_threadsafe_function::<RequestInput, (RequestInput,), Promise<Response>, Status, _, true, false, 0>(
            |ctx| Ok((ctx.value,)),
        )?;
    let m: Method = method
        .parse()
        .map_err(|_| napi::Error::new(Status::GenericFailure, format!("invalid method: {method}")))?;
    let mut routes = routes().lock().unwrap();
    push_kind(RouteKind::Fn(Arc::new(tsfn)), &path, &[m], &mut routes);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_pattern() {
        assert_eq!(translate_pattern("/users/:id"), "/users/<id>");
        assert_eq!(translate_pattern("/api/*"), "/api/<_..>");
    }

    #[test]
    fn test_route_creation() {
        let pattern = translate_pattern("/api/*");
        let res = rocket::http::uri::Origin::parse_route(&pattern);
        assert!(res.is_ok(), "failed to parse route: {:?}", res);
    }
}

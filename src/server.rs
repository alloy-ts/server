//! High-level, Bun-style HTTP server exposed to JavaScript via `#[napi]`.
//!
//! Everything AlloyScript code touches is generated here by the napi macros:
//!
//! ```ts
//! const server = serve({
//!   port: 3000,
//!   routes: {
//!     "/api/status": new Response("OK"),
//!     "/users/:id": req => new Response(`Hello ${req.params.id}`),
//!     "/api/posts": {
//!       GET: () => new Response("List posts"),
//!       POST: async req => Response.json({ created: true }),
//!     },
//!     "/api/*": Response.json({ message: "Not found" }, { status: 404 }),
//!     "/blog/hello": Response.redirect("/blog/hello/world"),
//!     "/favicon.ico": file("./favicon.ico"),
//!   },
//!   fetch(req) { return new Response("Not Found", { status: 404 }); },
//! });
//! console.log(server.url); // http://localhost:3000
//! ```
//!
//! `serve` receives the raw `options` object and introspects it at runtime.
//! A JS function can only be received by `#[napi]` as a *direct* parameter —
//! never nested — so rather than a typed struct we read `options` as a
//! [`JsObject`] and the `routes` table as a `HashMap<String, Unknown>`, then
//! branch on each value's type (function / `Response` / `file()` / per-method
//! map). No TypeScript glue required.
//!
//! ## Pattern translation
//!
//! Alloy/Bun use `:name` for a single-segment param and `*` for a multi-segment
//! glob; Rocket uses `<name>` and `<_..>`, so [`translate_pattern`] rewrites
//! them.

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Arc, Mutex};

use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
use napi_derive::napi;

use rocket::catcher::{self, Catcher};
use rocket::http::{Method, Status as RocketStatus};
use rocket::route::{Handler as RouteHandler, Outcome as RouteOutcome, Route};
use rocket::{Config, Data, Request, Response, Shutdown};

use crate::main::{build_request_payload, build_response, HandlerTsfn, JsHeader, ResponsePayload};

fn rocket_err<E: std::fmt::Display>(e: E) -> napi::Error {
    napi::Error::new(napi::Status::GenericFailure, e.to_string())
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

/// Translate an Alloy/Bun route pattern into Rocket route syntax.
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
            // Multi-segment glob: Alloy's `*` matches "all routes that start
            // with <prefix>" (e.g. `/api/*` covers `/api/foo/bar`).
            out.push_str("<_..>");
        } else {
            out.push(c);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// `Response` (public class) + `ResponseInit` + `FileRef`
// ---------------------------------------------------------------------------

/// Options for the `Response` constructor / `Response.json`.
#[napi(object)]
pub struct ResponseInit {
    pub status: Option<u32>,
    pub headers: Option<Vec<JsHeader>>,
}

/// Alloy's `Response`, mirroring the Bun/Web shape (`new Response(...)`,
/// `Response.json(...)`, `Response.redirect(...)`). Instances are structurally
/// `{ status, headers, body }`, so they deserialize straight into the internal
/// [`ResponsePayload`] wherever a response is expected.
#[napi]
pub struct Response {
    pub status: u32,
    pub headers: Vec<JsHeader>,
    pub body: Option<Buffer>,
}

#[napi]
impl Response {
    #[napi(constructor)]
    pub fn new(body: Option<String>, init: Option<ResponseInit>) -> Self {
        let status = init.as_ref().and_then(|i| i.status).unwrap_or(200);
        let headers = init.and_then(|i| i.headers).unwrap_or_default();
        let body = body.map(|s| Buffer::from(s.into_bytes()));
        Response {
            status,
            headers,
            body,
        }
    }

    /// `Response.json(data, init?)` — serialize `data` to JSON.
    #[napi]
    pub fn json(data: serde_json::Value, init: Option<ResponseInit>) -> Result<Self> {
        let bytes = serde_json::to_vec(&data)
            .map_err(|e| Error::new(Status::GenericFailure, format!("Response.json: {e}")))?;
        let mut headers = init
            .as_ref()
            .and_then(|i| i.headers.clone())
            .unwrap_or_default();
        headers.push(JsHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        });
        Ok(Response {
            status: init.and_then(|i| i.status).unwrap_or(200),
            headers,
            body: Some(Buffer::from(bytes)),
        })
    }

    /// `Response.redirect(url, status = 302)`.
    #[napi]
    pub fn redirect(url: String, status: Option<u32>) -> Self {
        Response {
            status: status.unwrap_or(302),
            headers: vec![JsHeader {
                name: "Location".to_string(),
                value: url,
            }],
            body: None,
        }
    }
}

/// Result of `file(path)` — a lazily-served static file reference.
#[napi(object)]
pub struct FileRef {
    pub path: String,
}

/// `file(path)` — lazy static file serving (MIME sniffed from extension).
#[napi]
pub fn file(path: String) -> FileRef {
    FileRef { path }
}

// ---------------------------------------------------------------------------
// Route handler bridge
// ---------------------------------------------------------------------------

/// What a mounted route does when hit.
enum RouteKind {
    /// Call a JS function: `(req) => Response | Promise<Response>`.
    Fn(Arc<HandlerTsfn>),
    /// Return a precomputed response.
    Static(ResponsePayload),
    /// Serve a file from disk, cached after the first read.
    File {
        path: String,
        cache: Arc<Mutex<Option<Vec<u8>>>>,
    },
}

#[derive(Clone)]
struct ServeHandler {
    kind: Arc<RouteKind>,
}

fn mime_for(path: &str) -> &'static str {
    match path.rsplit('.').next() {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") | Some("mjs") => "text/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("ico") => "image/x-icon",
        Some("txt") => "text/plain; charset=utf-8",
        Some("wasm") => "application/wasm",
        _ => "application/octet-stream",
    }
}

#[rocket::async_trait]
impl RouteHandler for ServeHandler {
    async fn handle<'r>(&self, req: &'r Request<'_>, data: Data<'r>) -> RouteOutcome<'r> {
        match &*self.kind {
            RouteKind::Fn(tsfn) => {
                let payload = build_request_payload(req, Some(data)).await;
                match tsfn.call_async(Ok((payload,))).await {
                    Ok(resp) => RouteOutcome::Success(build_response(resp)),
                    Err(_) => RouteOutcome::Error(RocketStatus::InternalServerError),
                }
            }
            RouteKind::Static(resp) => RouteOutcome::Success(build_response(resp.clone())),
            RouteKind::File { path, cache } => {
                let bytes = {
                    let guard = cache.lock().unwrap();
                    match guard.as_ref() {
                        Some(b) => b.clone(),
                        None => {
                            drop(guard);
                            match std::fs::read(path) {
                                Ok(b) => {
                                    *cache.lock().unwrap() = Some(b.clone());
                                    b
                                }
                                Err(_) => return RouteOutcome::Error(RocketStatus::NotFound),
                            }
                        }
                    }
                };
                RouteOutcome::Success(build_response(ResponsePayload {
                    status: Some(200),
                    headers: Some(vec![JsHeader {
                        name: "Content-Type".to_string(),
                        value: mime_for(path).to_string(),
                    }]),
                    body: Some(Buffer::from(bytes)),
                }))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Fallback catcher (the `fetch` handler)
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct ServeCatcher {
    tsfn: Option<Arc<HandlerTsfn>>,
}

#[rocket::async_trait]
impl catcher::Handler for ServeCatcher {
    async fn handle<'r>(&self, _status: RocketStatus, req: &'r Request<'_>) -> catcher::Result<'r> {
        match &self.tsfn {
            Some(tsfn) => {
                let payload = build_request_payload(req, None).await;
                match tsfn.call_async(Ok((payload,))).await {
                    Ok(resp) => Ok(build_response(resp)),
                    Err(_) => Err(RocketStatus::InternalServerError),
                }
            }
            None => Ok(build_response(ResponsePayload {
                status: Some(404),
                headers: None,
                body: None,
            })),
        }
    }
}

// ---------------------------------------------------------------------------
// Server handle returned to JS
// ---------------------------------------------------------------------------

#[napi]
pub struct Server {
    host: String,
    port: u16,
    unix: Option<String>,
    shutdown: Arc<Mutex<Option<Shutdown>>>,
}

#[napi]
impl Server {
    /// The bound TCP port.
    #[napi(getter)]
    pub fn port(&self) -> u16 {
        self.port
    }

    /// The canonical URL clients should connect to.
    #[napi(getter)]
    pub fn url(&self) -> String {
        match &self.unix {
            Some(path) => format!("unix://{path}"),
            None => format!("http://{}:{}", self.host, self.port),
        }
    }

    /// Begin graceful shutdown.
    #[napi]
    pub fn stop(&self) -> Result<()> {
        let guard = self.shutdown.lock().unwrap();
        match guard.as_ref() {
            Some(sh) => {
                sh.clone().notify();
                Ok(())
            }
            None => Err(Error::new(Status::GenericFailure, "server is not running")),
        }
    }
}

// ---------------------------------------------------------------------------
// Route-table introspection
// ---------------------------------------------------------------------------

/// Append one Rocket route per method for the given handler kind.
fn push_kind(kind: RouteKind, pattern: &str, methods: &[Method], out: &mut Vec<Route>) {
    let handler = ServeHandler {
        kind: Arc::new(kind),
    };
    let rocket_pattern = translate_pattern(pattern);
    for m in methods {
        out.push(Route::new(*m, &rocket_pattern, handler.clone()));
    }
}

/// Inspect one route value and append Rocket routes for it.
///
/// `value` is whatever the user put at `routes[pattern]` (or at a per-method
/// slot): a JS function, a `Response`/`file()` result, or a `{ METHOD: ... }`
/// map. The order of the typed casts matters: `FileRef` (has `path`) before
/// `ResponsePayload` (has `status`) before treating it as a method map.
fn mount_value(value: Unknown, pattern: &str, methods: &[Method], out: &mut Vec<Route>) -> Result<()> {
    match value.get_type()? {
        ValueType::Function => {
            let f: JsFunction = value.cast()?;
            let tsfn: HandlerTsfn = f.create_threadsafe_function(|ctx| Ok(ctx.value))?;
            push_kind(RouteKind::Fn(Arc::new(tsfn)), pattern, methods, out);
        }
        ValueType::Object => {
            let v = value.clone();
            if let Ok(fr) = v.clone().cast::<FileRef>() {
                push_kind(
                    RouteKind::File {
                        path: fr.path,
                        cache: Arc::new(Mutex::new(None)),
                    },
                    pattern,
                    methods,
                    out,
                );
            } else if let Ok(resp) = v.clone().cast::<ResponsePayload>() {
                push_kind(RouteKind::Static(resp), pattern, methods, out);
            } else {
                // Per-method map: { GET, POST, ... }.
                let map: HashMap<String, Unknown> = v.cast()?;
                for (method, method_value) in map {
                    let m: Method = method
                        .parse()
                        .map_err(|_| Error::new(Status::GenericFailure, format!("invalid method: {method}")))?;
                    mount_value(method_value, pattern, &[m], out)?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

/// Pick a free TCP port for `port: 0`. Binds a temporary socket, reads the
/// assigned port, then releases it. Tiny race before Rocket binds the same
/// port; acceptable for dev tooling. TODO: hand Rocket a pre-bound listener.
fn reserve_ephemeral_port() -> Result<u16> {
    let l = std::net::TcpListener::bind("127.0.0.1:0")
        .map_err(|e| Error::new(Status::GenericFailure, format!("ephemeral bind: {e}")))?;
    let port = l
        .local_addr()
        .map_err(|e| Error::new(Status::GenericFailure, format!("local_addr: {e}")))?
        .port();
    Ok(port)
}

// ---------------------------------------------------------------------------
// `serve` — the entry point
// ---------------------------------------------------------------------------

/// Start an `Alloy.serve` / `Bun.serve`-compatible HTTP server.
///
/// `options` is the raw JS object: `{ port?, hostname?, unix?, routes?, fetch? }`.
/// `routes` maps a pattern to either a function, a `Response`, `file(...)`,
/// or a `{ METHOD: function | Response | file(...) }` map. Static/redirect/
/// file routes are served entirely in Rust; dynamic routes and `fetch` are
/// bridged to JS. The call ignites synchronously so `server.port`/`server.url`
/// are valid immediately, then serves on a dedicated background runtime.
#[napi]
pub fn serve(options: JsObject) -> Result<Server> {
    // ---- resolve host / port ----
    let port: Option<u32> = options.get_named_property("port").unwrap_or(None);
    let hostname: Option<String> = options.get_named_property("hostname").unwrap_or(None);
    let unix: Option<String> = options.get_named_property("unix").unwrap_or(None);
    let hostname = hostname.unwrap_or_else(|| "0.0.0.0".to_string());
    let is_unix = unix.is_some();

    let port = match port {
        Some(0) | None => reserve_ephemeral_port()?,
        Some(p) => p as u16,
    };
    let host_for_url = if hostname == "0.0.0.0" || hostname == "::" {
        "localhost".to_string()
    } else {
        hostname.clone()
    };

    // ---- build Rocket config ----
    let mut cfg = Config::default();
    cfg.port = port;
    let ip: IpAddr = match hostname.parse::<IpAddr>() {
        Ok(ip) => ip,
        Err(_) if hostname == "localhost" => IpAddr::V4(Ipv4Addr::LOCALHOST),
        Err(_) => IpAddr::V4(Ipv4Addr::UNSPECIFIED),
    };
    cfg.address = ip;
    // Rocket 0.5 has no stable unix-socket config field; when `unix` is set we
    // currently still listen on TCP. TODO: wire a custom `Listener`.
    let _ = is_unix;

    let mut rocket = rocket::custom(cfg);

    // ---- mount routes from the raw `routes` table ----
    let mut routes: Vec<Route> = Vec::new();
    if let Some(table) = options
        .get_named_property::<HashMap<String, Unknown>>("routes")
        .unwrap_or(None)
    {
        for (pattern, value) in table {
            mount_value(value, &pattern, ALL_METHODS, &mut routes)?;
        }
    }
    if !routes.is_empty() {
        rocket = rocket.mount("/", routes);
    }

    // ---- fallback catcher (the `fetch` handler) ----
    let fetch_tsfn: Option<Arc<HandlerTsfn>> =
        match options.get_named_property::<JsFunction>("fetch").unwrap_or(None) {
            Some(f) => Some(Arc::new(f.create_threadsafe_function(|ctx| Ok(ctx.value))?)),
            None => None,
        };
    rocket = rocket.register("/", vec![Catcher::new(Some(404), ServeCatcher { tsfn: fetch_tsfn })]);

    // ---- ignite (resolve config) + launch on a dedicated runtime ----
    let rt = rocket::tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| Error::new(Status::GenericFailure, format!("tokio runtime: {e}")))?;

    let rocket = rt.block_on(rocket.ignite()).map_err(rocket_err)?;
    let shutdown = rocket.shutdown();
    let shutdown_arc = Arc::new(Mutex::new(Some(shutdown)));

    std::thread::spawn(move || {
        let _ = rt.block_on(rocket.launch());
    });

    Ok(Server {
        host: host_for_url,
        port,
        unix: unix.clone(),
        shutdown: shutdown_arc,
    })
}

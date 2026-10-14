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
//! The public surface below (`serve`, `Response`, `file`, `response`,
//! `responseJson`, `responseRedirect`, `register_route`, `Server`) is the single
//! source of truth: `npm run build` generates `index.js` / `index.d.ts` from
//! these `#[napi]` exports, so no hand-written `.ts` shim (e.g. a former
//! `src/serve.ts`) is needed.
//!
//! Note on `Response`: it is a `#[napi(object)]` (plain data) rather than a
//! `#[napi]` class because napi class fields must support `&mut` serialization,
//! which complex types (`HashMap`, nested objects) do not. The constructor /
//! static helpers (`response`, `responseJson`, `responseRedirect`) are exposed as
//! free `#[napi]` functions that return this object, keeping the Bun-style
//! `Response.json(data)` / `Response.redirect(url)` ergonomics available to the
//! TS glue layer.

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
use rocket::route::Route;
use crate::handler::JsHandler;
use rocket::{Data, Shutdown};

/// Convenience alias for the threadsafe function each JS route handler is
/// wrapped in. The JS function receives a [`RequestInput`] and must resolve a
/// [`Promise<Response>`] (i.e. be `async` and return a `Response`).
///
/// The `create_threadsafe_function` generic order is
/// `<T, NewArgs, Return, ErrorStatus, F, ES, Weak, MaxQueueSize>`; the closure
/// returns `NewArgs` (a 1-tuple of the request), while `Return` is the JS
/// function's resolved value (`Promise<Response>`, whose `FromNapiValue` awaits
/// the JS promise for us).
type HandlerTsfn =
    ThreadsafeFunction<RequestInput, Promise<Response>, (RequestInput,), Status, true, false, 0>;

/// Convert a `napi::Unknown` (received as a `#[napi]` parameter or parsed out of
/// a JS object) into a `napi::JsFunction` so we can call
/// `create_threadsafe_function`.
///
/// `JsFunction` does not implement `FromNapiValue`, so `Unknown::cast` cannot
/// produce it. The two types share the identical `NapiValueInner` layout (an
/// env pointer + a value pointer), so a bit-copy is sound: the napi handles are
/// cloned into the resulting `ThreadsafeFunction`, so dropping the source
/// `Unknown` is safe.
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

/// The `Response` value a JS handler returns. It is a plain `#[napi(object)]`
/// (see the module docs for why it is not a class) carrying the same `status` /
/// `headers` / `body` shape that `response`, `responseJson`, `responseRedirect`
/// produce.
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
pub(crate) enum RouteKind {
    Fn(Arc<HandlerTsfn>),
    Static(Arc<Response>),
    File {
        path: String,
    },
}

/// Extract the request data Rocket matched into the JS-facing [`RequestInput`],
/// including any `:param` / `<param>` path segments.
pub(crate) async fn build_request(request: &Request<'_>, data: Data<'_>, pattern: &str) -> RequestInput {
    let method = request.method().as_str().to_string();
    let uri = request.uri();
    let path = uri.path().to_string();
    let query = uri.query().map(|q| q.to_string()).unwrap_or_default();
    let headers = request
        .headers()
        .iter()
        .map(|h| (h.name.as_str().to_string(), h.value().to_string()))
        .collect();

    // Read up to 1 MiB of body. Larger bodies are truncated for the MVP; raise
    // the `ByteUnit` limit (or stream) when needed.
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
            // `<_..>` catch-all -> skip; `<name>` or `<name..>` -> capture.
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
            // Multi-segment glob: Alloy's `*` matches "all routes that start
            // with <prefix>" (e.g. `/api/*` covers `/api/foo/bar`).
            out.push_str("<_..>");
        } else {
            out.push(c);
        }
    }
    out
}

/// Build a Rocket `Response` from a JS handler's `Response`, mirroring the
/// `Responder` step in Rocket's generated code.
pub(crate) fn build_response(payload: &Response) -> RocketResponse<'static> {
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
///
/// `value` is whatever the user put at `routes[pattern]` (or at a per-method
/// slot): a JS function, a `Response`/`file()` result, or a `{ METHOD: ... }`
/// map. The order of the typed casts matters: `StaticFile` (has `path`) before
/// `Response` (has `status`) before treating it as a method map.
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
                f.create_threadsafe_function::<RequestInput, (RequestInput,), Promise<Response>, Status, _, true, false, 0>(
                    |ctx| Ok((ctx.value,)),
                )?;
            push_kind(RouteKind::Fn(Arc::new(tsfn)), pattern, methods, out);
        }
        ValueType::Object => {
            let v = value.clone();
            if let Ok(fr) = unsafe { v.clone().cast::<StaticFile>() } {
                push_kind(
                    RouteKind::File {
                        path: fr.path,
                    },
                    pattern,
                    methods,
                    out,
                );
            } else if let Ok(resp) = unsafe { v.clone().cast::<Response>() } {
                push_kind(RouteKind::Static(Arc::new(resp)), pattern, methods, out);
            } else {
                // Per-method map: `{ GET, POST, ... }`.
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
        _ => {}
    }
    Ok(())
}

/// Append one [`RouteKind`] for every method in `methods`, translating the
/// Alloy/Bun pattern into Rocket syntax.
fn push_kind(kind: RouteKind, pattern: &str, methods: &[Method], out: &mut Vec<Route>) {
    let handler = JsHandler(Arc::new(kind));
    let rocket_pattern = translate_pattern(pattern);
    for m in methods {
        out.push(Route::new(*m, &rocket_pattern, handler.clone()));
    }
}

/// Shared shutdown handle, filled by the launch fairing and drained by
/// [`Server::stop`].
type ShutdownSlot = Arc<Mutex<Option<Shutdown>>>;
static SHUTDOWN: OnceLock<ShutdownSlot> = OnceLock::new();

/// NAPI equivalent of `Alloy.serve({ routes, fetch, port, hostname })` + the
/// launch step. Parses the JS `routes` table, builds a `Rocket<Build>` from every
/// route, spawns it on the tokio runtime, and returns a [`Server`] handle
/// immediately (mirroring Bun's synchronous `serve` return).
///
/// `routes` and `fetch` are taken as direct parameters (a JS object / function
/// cannot be a `#[napi(object)]` struct field); `options` carries the scalar
/// config (`port`, `hostname`, `unix`).
#[napi]
pub fn serve(
    routes: Option<Unknown<'_>>,
    fetch: Option<Unknown<'_>>,
    options: Option<ServeOptions>,
) -> napi::Result<Server> {
    let options = options.unwrap_or_else(|| ServeOptions {
        port: None,
        hostname: None,
        unix: None,
    });
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

    let mut out: Vec<Route> = Vec::new();
    if let Some(table) = routes {
        let table: Object = unsafe { table.cast()? };
        let names = table.get_property_names()?;
        let len = names.get_array_length_unchecked()?;
        for i in 0..len {
            let pattern: String = names.get_element(i)?;
            let value: Unknown<'_> = table.get_named_property(&pattern)?;
            mount_value(value, &pattern, ALL_METHODS, &mut out)?;
        }
    }

    // `fetch` fallback: a catch-all for every method on any unmatched path.
    if let Some(fetch) = fetch {
        let f = to_js_function(fetch);
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
    SHUTDOWN.get_or_init(|| slot.clone());

    let rocket = rocket::build()
        .configure(config)
        .mount("/", out)
        .attach(AdHoc::on_liftoff("capture-shutdown", move |orbit| {
            Box::pin(async move {
                *slot.lock().unwrap() = Some(orbit.shutdown());
            })
        }));
    // Run the server on its own dedicated tokio runtime in a background thread.
    // `serve` is invoked synchronously from the Node event loop (no surrounding
    // tokio context), so we cannot use `rocket::tokio::spawn` here — that would
    // panic with "no reactor running". A standalone runtime guarantees Rocket has
    // the executor it needs while keeping the JS call non-blocking.
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
    })
}

/// Options for [`serve`] (scalar config only; `routes`/`fetch` are parameters).
#[napi(object)]
pub struct ServeOptions {
    pub port: Option<u32>,
    pub hostname: Option<String>,
    pub unix: Option<String>,
}

/// Handle returned by [`serve`]; exposes `url`, `port`, and `stop()`.
#[napi]
pub struct Server {
    pub url: String,
    pub port: u32,
}

#[napi]
impl Server {
    /// Gracefully stop the server (waits for in-flight requests).
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
// Low-level registry API: `register_route` for the codegen/decorator path that
// builds routes one-at-a-time before booting.
// ---------------------------------------------------------------------------

static ROUTES: OnceLock<Mutex<Vec<Route>>> = OnceLock::new();

fn routes() -> &'static Mutex<Vec<Route>> {
    ROUTES.get_or_init(|| Mutex::new(Vec::new()))
}

/// NAPI equivalent of `#[get("/p")] fn ..` + adding it to a `routes!` list.
///
/// The `handler` JS function must return a `Promise<Response>` (i.e. be an
/// `async` function returning a `Response`). Route definitions accumulate until
/// [`serve`] is called, which mounts them alongside the `routes` table.
#[napi]
pub fn register_route(method: String, path: String, handler: Unknown<'_>) -> napi::Result<()> {
    let f = to_js_function(handler);
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

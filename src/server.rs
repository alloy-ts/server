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
//! the JS return value, mirroring the "generated handler validates args, calls
//! the function, uses the `Responder` to build a `Response`" semantics described
//! in `rocket_codegen`'s route-attribute docs.
//!
//! The public surface below (`serve`, `Response`, `file`, `register_route`,
//! `launch`, `Server`) is the single source of truth: `npm run build` generates
//! `index.js` / `index.d.ts` from these `#[napi]` exports, so no hand-written
//! `.ts` shim (e.g. a former `src/serve.ts`) is needed.

use std::collections::HashMap;
use std::io::Cursor;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Mutex, OnceLock};

use napi::bindgen_prelude::{Buffer, JsFunction, Object, Status};
use napi::threadsafe_function::{ThreadsafeCallContext, ThreadsafeFunction};
use napi::JsUnknown;
use napi::Promise;
use napi::ValueType;

use rocket::config::Config;
use rocket::data::ByteUnit;
use rocket::http::{Header, Method, Status as RocketStatus};
use rocket::outcome::Outcome;
use rocket::request::Request;
use rocket::response::Response as RocketResponse;
use rocket::route::{Handler, Route};
use rocket::{Data, Shutdown};

/// Wire type sent from Rust -> JS for every request. Mirrors the request data
/// Rocket's generated route handler would have extracted (method, uri, headers,
/// body, and the `:param` / `<param>` segments) before calling the user fn.
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

/// The `Response` value a JS handler returns (or a static route value). Mirrors
/// the value Rocket's `Responder` trait would turn into a `Response`.
#[napi]
#[derive(Clone)]
pub struct Response {
    pub status: u32,
    pub headers: HashMap<String, String>,
    pub body: Option<Buffer>,
}

#[napi]
impl Response {
    /// `new Response(body, init)` — `body` may be a `string`, `Uint8Array`, or
    /// `null` (matching the Bun/Web `Response` shape).
    #[napi(constructor)]
    pub fn new(body: Option<JsUnknown>, init: Option<ResponseInit>) -> napi::Result<Self> {
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

    /// `Response.json(data, init)` — stringifies `data` as JSON with the
    /// `application/json` content type.
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

    /// `Response.redirect(url, status)` — `status` defaults to 302.
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

/// A JS route handler, callable from any Rocket worker thread. The `Return` type
/// is `Promise<Response>` so async TS handlers (`async (req) => ...`) are awaited
/// on the Rust side via [`ThreadsafeFunction::call_async`].
type RouteHandlerFn =
    ThreadsafeFunction<RequestInput, Promise<Response>, RequestInput, Status, true, false, 0>;

/// The resolved intent of one route entry after parsing the JS `routes` table.
#[derive(Clone)]
enum RouteHandler {
    Dynamic(RouteHandlerFn),
    Static(Response),
    File(String),
}

/// A Rocket `Handler` that dispatches to one of the three [`RouteHandler`] forms.
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
                let output = match tsfn.call_async(Ok(input)).await {
                    // `Promise::await` already formatted any JS rejection into an error;
                    // map both call and await failures to a 500, which Rocket's error
                    // catcher turns into the response.
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
        .map(|h| (h.name().as_str().to_string(), h.value().to_string()))
        .collect();

    // Read up to 1 MiB of body. Larger bodies are truncated for the MVP; raise
    // the `ByteUnit` limit (or stream) when needed.
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

/// Build a Rocket `Response` from a JS handler's `Response` return value,
/// mirroring the `Responder` step in Rocket's generated code.
fn build_response(out: Response) -> RocketResponse<'static> {
    let mut builder = RocketResponse::build();
    if let Some(status) = RocketStatus::from_code(out.status as u16) {
        builder.status(status);
    }
    for (name, value) in out.headers {
        builder.header(Header::new(name, value));
    }
    if let Some(body) = out.body {
        let bytes: Vec<u8> = body.as_ref().to_vec();
        builder.sized_body(bytes.len(), Cursor::new(bytes));
    }
    builder.finalize()
}

/// Options for [`serve`]; mirrors the Bun/Alloy `serve({ ... })` shape.
#[napi(object)]
pub struct ServeOptions {
    pub port: Option<u32>,
    pub hostname: Option<String>,
    pub unix: Option<String>,
    pub routes: Option<Object>,
    pub fetch: Option<JsFunction>,
}

const ALL_METHODS: &[&str] = &["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];

/// Parse one `routes` table value into `(method, handler)` pairs. `None` method
/// means "register for every method".
fn collect_routes(value: JsUnknown) -> napi::Result<Vec<(Option<String>, RouteHandler)>> {
    match value.get_type()? {
        ValueType::Function => {
            let f: JsFunction = unsafe { value.cast() }?;
            let tsfn = f
                .create_threadsafe_function::<RequestInput, RequestInput, Promise<Response>>(
                |ctx| Ok(ctx.value),
            )?;
            Ok(vec![(None, RouteHandler::Dynamic(tsfn))])
        }
        ValueType::Object => {
            // A `file(...)` marker is a plain `{ path }` object.
            if let Ok(file) = unsafe { value.cast::<StaticFile>() } {
                return Ok(vec![(None, RouteHandler::File(file.path))]);
            }
            // A `Response` instance created via `new Response(...)`.
            if let Ok(resp) = unsafe { value.cast::<Response>() } {
                return Ok(vec![(None, RouteHandler::Static(resp))]);
            }
            // Otherwise it's a per-method map: `{ GET, POST, ... }`.
            let obj: Object = unsafe { value.cast() }?;
            let names = obj.get_property_names()?;
            let len = names.get_array_length()?;
            let mut out = Vec::new();
            for i in 0..len {
                let method: String = names.get_element(i)?;
                let v: JsUnknown = obj.get_named_property(&method)?;
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

/// Captured `Shutdown` handle so `Server::stop` can terminate the running server.
static SHUTDOWN: OnceLock<Mutex<Option<Shutdown>>> = OnceLock::new();

/// NAPI equivalent of `Alloy.serve({ routes, fetch, port, hostname })` + the
/// launch step. Parses the JS `routes` table, builds a `Rocket<Build>` from every
/// route, spawns it on the tokio runtime, and resolves immediately with a
/// [`Server`] handle (mirroring Bun's synchronous `serve` return).
#[napi]
pub async fn serve(options: ServeOptions) -> napi::Result<Server> {
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
            let value: JsUnknown = routes.get_named_property(&pattern)?;
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

    // `fetch` fallback: a catch-all for every method on any unmatched path.
    if let Some(fetch) = options.fetch {
        let tsfn = fetch
            .create_threadsafe_function::<RequestInput, RequestInput, Promise<Response>>(
            |ctx| Ok(ctx.value),
        )?;
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

    let mut rocket = rocket::build().configure(config).mount("/", rocket_routes);
    let shutdown = rocket.shutdown();
    *SHUTDOWN.get_or_init(|| Mutex::new(None)).lock().unwrap() = Some(shutdown);
    rocket::tokio::spawn(async move {
        let _ = rocket.launch().await;
    });

    Ok(Server {
        url,
        port: port as u32,
    })
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
        if let Some(shutdown) = SHUTDOWN.lock().unwrap().take() {
            shutdown.shutdown();
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Low-level registry API: `register_route` + `launch` for the codegen/decorator
// path that builds routes one-at-a-time before booting.
// ---------------------------------------------------------------------------

struct RouteSpec {
    method: String,
    path: String,
    handler: RouteHandler,
}

static ROUTES: OnceLock<Mutex<Vec<RouteSpec>>> = OnceLock::new();

fn routes() -> &'static Mutex<Vec<RouteSpec>> {
    ROUTES.get_or_init(|| Mutex::new(Vec::new()))
}

/// NAPI equivalent of `#[get("/p")] fn ..` + adding it to a `routes!` list.
///
/// The `handler` JS function must return a `Promise<Response>` (i.e. be an
/// `async` function returning a `Response`). Route definitions accumulate until
/// [`launch`] is called, which builds and boots the Rocket instance.
#[napi]
pub fn register_route(method: String, path: String, handler: JsFunction) -> napi::Result<()> {
    let tsfn = handler
        .create_threadsafe_function::<RequestInput, RequestInput, Promise<Response>>(|ctx| {
            Ok(ctx.value)
        })?;
    routes()
        .lock()
        .unwrap()
        .push(RouteSpec {
            method,
            path,
            handler: RouteHandler::Dynamic(tsfn),
        });
    Ok(())
}

/// NAPI equivalent of `#[launch]`: build the `Rocket<Build>` from every route
/// registered via [`register_route`], mount them, and boot the server. Resolves
/// once the server shuts down.
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
                pattern: translated,
            },
        ));
    }

    rocket::build()
        .mount("/", rocket_routes)
        .launch()
        .await
        .map(|_orbit| ())
        .map_err(|e| napi::Error::new(Status::GenericFailure, e.to_string()))
}

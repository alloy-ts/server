//! NAPI bindings that let Node.js drive a Rocket server.
//!
//! This mirrors Rocket's server lifecycle (`rocket/src/server.rs`): building a
//! `Rocket<Build>`, mounting [`Route`]s and [`Catcher`]s, igniting, launching,
//! and graceful shutdown with the grace/mercy phases. It is the NAPI equivalent
//! of `#[launch]` / `#[rocket::main]`:
//!
//! ```rust,no_run
//! #[rocket::main]
//! async fn main() -> Result<(), rocket::Error> {
//!     let rocket = rocket::build().ignite().await?;
//!     rocket.launch().await?;
//!     Ok(())
//! }
//! ```
//!
//! ...where the `async fn` is instead driven by NAPI's tokio runtime. JS handlers
//! are bridged into Rocket via [`rocket::route::Handler`] / [`rocket::catcher::Handler`]
//! implementations that call a JS callback through a
//! [`napi::threadsafe_function::ThreadsafeFunction`].
//!
//! Note: a JS function can only be received as a *direct* `#[napi]` parameter
//! (not nested in an object), so routes/catchers are mounted one handler at a
//! time via [`Rocket::mount`] / [`Rocket::register`].

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
#[allow(deprecated)]
use napi::{JsFunction, Unknown};
use napi_derive::napi;

use rocket::catcher::{self, Catcher};
use rocket::http::{Method, Status as RocketStatus};
use rocket::route::{Handler as RouteHandler, Outcome as RouteOutcome, Route};
use rocket::tokio::io::AsyncReadExt;
use rocket::{Build, Config, Data, Request, Response, Rocket as RocketServer, Shutdown};
use rocket::config::{Ident, LogLevel, Shutdown as ShutdownConfig};

/// Convert any displayable Rocket error into an NAPI error.
fn rocket_err<E: std::fmt::Display>(e: E) -> napi::Error {
    napi::Error::new(napi::Status::GenericFailure, e.to_string())
}

// ---------------------------------------------------------------------------
// JS <-> Rust translation
// ---------------------------------------------------------------------------

/// A request handed to a JS handler.
#[napi(object)]
pub struct RequestPayload {
    pub method: String,
    pub path: String,
    pub query: Option<String>,
    pub headers: Vec<JsHeader>,
    /// Path parameters extracted from the route pattern (e.g. `req.params.id`).
    pub params: Option<HashMap<String, String>>,
    /// Raw request body, or `undefined` when the request carries no body (e.g.
    /// during error handling where Rocket does not provide one).
    pub body: Option<Buffer>,
}

#[napi(object)]
pub struct JsHeader {
    pub name: String,
    pub value: String,
}

/// A response returned from a JS handler.
#[napi(object)]
pub struct ResponsePayload {
    pub status: Option<u32>,
    pub headers: Option<Vec<JsHeader>>,
    pub body: Option<Buffer>,
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
            if let Some(name) = inner.strip_suffix("..>") {
                if name != "_" && idx < a_segs.len() {
                    let rest = a_segs[idx..].join("/");
                    params.insert(name.to_string(), rest);
                }
            } else if let Some(name) = inner.strip_suffix('>') {
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

/// Build a `RequestPayload` from a Rocket request. The body is read fully when
/// `data` is supplied; catchers receive `None` and produce no body.
pub(crate) async fn build_request_payload(req: &Request<'_>, data: Option<Data<'_>>) -> RequestPayload {
    let uri = req.uri();
    let method = req.method().as_str().to_string();
    let path = uri.path().to_string();
    let query = uri.query().map(|q| q.to_string());

    let headers: Vec<JsHeader> = req
        .headers()
        .iter()
        .map(|h| JsHeader {
            name: h.name.as_str().to_string(),
            value: h.value.to_string(),
        })
        .collect();

    let body = match data {
        Some(data) => {
            let mut stream = data.open(rocket::data::ByteUnit::max_value());
            let mut buf = Vec::new();
            let _ = stream.read_to_end(&mut buf).await;
            if buf.is_empty() {
                None
            } else {
                Some(Buffer::from(buf))
            }
        }
        None => None,
    };

    let params: HashMap<String, String> = req
        .route()
        .map(|route| {
            let pattern = route.uri.to_string();
            let path = req.uri().path().to_string();
            extract_params(&pattern, &path)
        })
        .unwrap_or_default();

    RequestPayload {
        method,
        path,
        query,
        headers,
        params: if params.is_empty() { None } else { Some(params) },
        body,
    }
}

/// Build a Rocket `Response` from a JS handler's return value.
pub(crate) fn build_response(resp: ResponsePayload) -> Response<'static> {
    let status =
        RocketStatus::from_code(resp.status.unwrap_or(200) as u16).unwrap_or(RocketStatus::Ok);
    let mut builder = Response::build();
    builder.status(status);

    if let Some(headers) = resp.headers {
        for h in headers {
            builder.raw_header(h.name, h.value);
        }
    }

    if let Some(body) = resp.body {
        let bytes: Vec<u8> = body.to_vec();
        builder.sized_body(bytes.len(), std::io::Cursor::new(bytes));
    }

    builder.finalize()
}

type HandlerTsfn =
    ThreadsafeFunction<(RequestPayload,), Either<ResponsePayload, Promise<ResponsePayload>>>;

fn to_js_function(handler: Unknown) -> napi::Result<JsFunction> {
    if handler.get_type()? != ValueType::Function {
        return Err(napi::Error::new(Status::InvalidArg, "expected a function"));
    }
    #[allow(deprecated)]
    unsafe { Ok(std::mem::transmute_copy::<Unknown, JsFunction>(&handler)) }
}

// ---------------------------------------------------------------------------
// Route handler bridge
// ---------------------------------------------------------------------------

/// A `rocket::route::Handler` that forwards to a JS callback.
#[derive(Clone)]
struct JsRouteHandler {
    tsfn: Arc<HandlerTsfn>,
}

#[rocket::async_trait]
impl RouteHandler for JsRouteHandler {
    async fn handle<'r>(&self, req: &'r Request<'_>, data: Data<'r>) -> RouteOutcome<'r> {
        let payload = build_request_payload(req, Some(data)).await;
        match self.tsfn.call_async(Ok((payload,))).await {
            Ok(Either::A(resp)) => RouteOutcome::Success(build_response(resp)),
            Ok(Either::B(promise)) => match promise.await {
                Ok(resp) => RouteOutcome::Success(build_response(resp)),
                Err(_) => RouteOutcome::Error(RocketStatus::InternalServerError),
            },
            Err(_) => RouteOutcome::Error(RocketStatus::InternalServerError),
        }
    }
}

// ---------------------------------------------------------------------------
// Catcher handler bridge
// ---------------------------------------------------------------------------

/// A `rocket::catcher::Handler` that forwards to a JS callback.
#[derive(Clone)]
struct JsCatcherHandler {
    tsfn: Arc<HandlerTsfn>,
}

#[rocket::async_trait]
impl catcher::Handler for JsCatcherHandler {
    async fn handle<'r>(&self, _status: RocketStatus, req: &'r Request<'_>) -> catcher::Result<'r> {
        let payload = build_request_payload(req, None).await;
        match self.tsfn.call_async(Ok((payload,))).await {
            Ok(Either::A(resp)) => Ok(build_response(resp)),
            Ok(Either::B(promise)) => match promise.await {
                Ok(resp) => Ok(build_response(resp)),
                Err(_) => Err(RocketStatus::InternalServerError),
            },
            Err(_) => Err(RocketStatus::InternalServerError),
        }
    }
}

// ---------------------------------------------------------------------------
// JS-facing types
// ---------------------------------------------------------------------------

#[napi(object)]
pub struct ShutdownOptions {
    pub grace: Option<u32>,
    pub mercy: Option<u32>,
    pub force: Option<bool>,
}

/// Rocket configuration, mirroring `rocket::Config` (a `Figment` provider).
/// Any field left unset falls back to Rocket's defaults.
#[napi(object)]
pub struct RocketConfig {
    pub address: Option<String>,
    pub port: Option<u16>,
    pub workers: Option<u32>,
    pub max_blocking: Option<u32>,
    pub keep_alive: Option<u32>,
    pub ident: Option<String>,
    pub log_level: Option<String>,
    pub shutdown: Option<ShutdownOptions>,
}

/// A handle to a Rocket server, mirroring `#[launch]` / `#[rocket::main]`.
#[napi]
pub struct Rocket {
    inner: Arc<Mutex<Option<RocketServer<Build>>>>,
    shutdown: Arc<Mutex<Option<Shutdown>>>,
}

#[napi]
impl Rocket {
    /// Create a new Rocket instance from the given configuration.
    #[napi(constructor)]
    pub fn new(config: Option<RocketConfig>) -> Result<Self> {
        let mut cfg = Config::default();

        if let Some(c) = config {
            if let Some(addr) = c.address {
                cfg.address = addr
                    .parse()
                    .map_err(|e| Error::new(Status::GenericFailure, format!("invalid address: {e}")))?;
            }
            if let Some(p) = c.port {
                cfg.port = p;
            }
            if let Some(w) = c.workers {
                cfg.workers = w as usize;
            }
            if let Some(mb) = c.max_blocking {
                cfg.max_blocking = mb as usize;
            }
            if let Some(ka) = c.keep_alive {
                cfg.keep_alive = ka;
            }
            if let Some(ident) = c.ident {
                cfg.ident = Ident::try_new(ident)
                    .map_err(|e| Error::new(Status::GenericFailure, e))?;
            }
            if let Some(ll) = c.log_level {
                cfg.log_level = ll
                    .parse::<LogLevel>()
                    .map_err(|e| Error::new(Status::GenericFailure, format!("invalid log_level: {e}")))?;
            }
            if let Some(sh) = c.shutdown {
                cfg.shutdown = ShutdownConfig {
                    grace: sh.grace.unwrap_or(cfg.shutdown.grace),
                    mercy: sh.mercy.unwrap_or(cfg.shutdown.mercy),
                    force: sh.force.unwrap_or(cfg.shutdown.force),
                    ..Default::default()
                };
            }
        }

        Ok(Self {
            inner: Arc::new(Mutex::new(Some(rocket::custom(cfg)))),
            shutdown: Arc::new(Mutex::new(None)),
        })
    }

    /// Mount a single `handler` for `method` at `path` under `base`.
    /// Equivalent to `rocket.mount(base, vec![Route::new(method, path, handler)])`.
    ///
    /// `handler` is `(err: null, req: RequestPayload) => ResponsePayload | Promise<ResponsePayload>`.
    #[napi]
    #[allow(deprecated)]
    pub fn mount(&self, base: String, method: String, path: String, handler: Unknown) -> Result<()> {
        let method: Method = method
            .parse()
            .map_err(|_| Error::new(Status::GenericFailure, format!("invalid method: {method}")))?;
        let tsfn: HandlerTsfn = to_js_function(handler)?.create_threadsafe_function(|ctx| Ok(ctx.value))?;
        let route = Route::new(method, &path, JsRouteHandler { tsfn: Arc::new(tsfn) });

        let mut guard = self.inner.lock().unwrap();
        let rocket = guard
            .take()
            .ok_or_else(|| Error::new(Status::GenericFailure, "server already launched"))?;
        *guard = Some(rocket.mount(&base, vec![route]));
        Ok(())
    }

    /// Register a single catcher `handler`. `code` is the HTTP status it handles
    /// (in `[400, 600)`); pass `None` for the default catcher.
    /// Equivalent to `rocket.register(base, vec![Catcher::new(code, handler)])`.
    #[napi]
    #[allow(deprecated)]
    pub fn register(&self, base: String, code: Option<u32>, handler: Unknown) -> Result<()> {
        let code: Option<u16> = code.map(|c| c as u16);
        let tsfn: HandlerTsfn = to_js_function(handler)?.create_threadsafe_function(|ctx| Ok(ctx.value))?;
        let catcher = Catcher::new(code, JsCatcherHandler { tsfn: Arc::new(tsfn) });

        let mut guard = self.inner.lock().unwrap();
        let rocket = guard
            .take()
            .ok_or_else(|| Error::new(Status::GenericFailure, "server already launched"))?;
        *guard = Some(rocket.register(&base, vec![catcher]));
        Ok(())
    }

    /// Ignite and launch this server, resolving when the server shuts down.
    ///
    /// This mirrors the generated `main` from `#[launch]`:
    ///
    /// ```rust,no_run
    /// #[rocket::main]
    /// async fn main() {
    ///     let _ = rocket().launch().await;
    /// }
    /// ```
    #[napi]
    pub async fn launch(&self) -> Result<()> {
        let rocket = {
            let mut guard = self
                .inner
                .lock()
                .map_err(|_| Error::new(Status::GenericFailure, "lock poisoned"))?;
            guard.take().ok_or_else(|| {
                Error::new(Status::GenericFailure, "server already launched or not configured")
            })?
        };

        let rocket = rocket.ignite().await.map_err(rocket_err)?;
        let shutdown = rocket.shutdown();
        *self.shutdown.lock().unwrap() = Some(shutdown);
        rocket.launch().await.map_err(rocket_err)?;
        Ok(())
    }

    /// Begin graceful shutdown. Equivalent to `Shutdown::notify()`.
    #[napi]
    pub fn shutdown(&self) -> Result<()> {
        let guard = self.shutdown.lock().unwrap();
        match guard.as_ref() {
            Some(shutdown) => {
                shutdown.clone().notify();
                Ok(())
            }
            None => Err(Error::new(Status::GenericFailure, "server is not running")),
        }
    }
}

/// Convenience mirror of `#[launch]`: build (with optional config), ignite,
/// launch, and resolve once the server shuts down.
#[napi]
pub async fn launch(config: Option<RocketConfig>) -> Result<()> {
    Rocket::new(config)?.launch().await
}

/// Mirror of `#[rocket::main]`'s inspectable form: ignite, launch, resolve.
#[napi]
pub async fn rocket_main(config: Option<RocketConfig>) -> Result<()> {
    Rocket::new(config)?.launch().await
}

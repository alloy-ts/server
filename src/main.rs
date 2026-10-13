//! NAPI bindings that let Node.js drive a service built on the underlying framework.
//!
//! This mirrors the underlying framework's service lifecycle: building the initial
//! configuration, mounting [`Route`]s and [`Catcher`]s, igniting, launching, and
//! graceful shutdown with the grace/mercy phases. It is the NAPI equivalent of the
//! framework's own launch entrypoint.
//!
//! JS handlers are bridged into the underlying framework via its route and catcher
//! `Handler` traits, implemented to call a JS callback through a
//! [`napi::threadsafe_function::ThreadsafeFunction`].
//!
//! Note: a JS function can only be received as a *direct* `#[napi]` parameter
//! (not nested in an object), so routes/catchers are mounted one handler at a
//! time via the `mount` / `register` methods.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
#[allow(deprecated)]
use napi::{JsFunction, Unknown};
use napi_derive::napi;

use service::catcher::{self, Catcher};
use service::http::{Method, Status as ServiceStatus};
use service::route::{Handler as RouteHandler, Outcome as RouteOutcome, Route};
use service::tokio::io::AsyncReadExt;
use service::{Build, Config, Data, Request, Response, Service as Svc, Shutdown};
use service::config::{Ident, LogLevel, Shutdown as ShutdownConfig};

/// Convert any displayable error from the underlying framework into an NAPI error.
fn service_err<E: std::fmt::Display>(e: E) -> napi::Error {
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
    /// during error handling where the underlying framework does not provide one).
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

/// Build a `RequestPayload` from a request handled by the underlying framework.
/// The body is read fully when `data` is supplied; catchers receive `None` and
/// produce no body.
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
            let mut stream = data.open(service::data::ByteUnit::max_value());
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
            route
                .param_names()
                .iter()
                .filter_map(|name| req.raw_param(name).map(|v| (name.to_string(), v.to_string())))
                .collect()
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

/// Build a `Response` from a JS handler's return value.
pub(crate) fn build_response(resp: ResponsePayload) -> Response<'static> {
    let status =
        ServiceStatus::from_code(resp.status.unwrap_or(200) as u16).unwrap_or(ServiceStatus::Ok);
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

    builder.finalise()
}

type HandlerTsfn = ThreadsafeFunction<(RequestPayload,), ResponsePayload>;

/// Bridge a JS `Unknown` into a `JsFunction`.
///
/// In napi 3.14, the `#[napi]`-accepted `Function`/`Unknown` types and the
/// `JsFunction` type share the identical `NapiValueInner` representation
/// (an env pointer + a value pointer), but only `JsFunction` exposes
/// `create_threadsafe_function` with a typed return value. This copy is sound
/// because the layout is identical; the underlying `napi` handles are cloned
/// into the new `ThreadsafeFunction`, so dropping the original `Unknown` is safe.
#[allow(deprecated)]
fn to_js_function(handler: Unknown) -> JsFunction {
    debug_assert_eq!(
        std::mem::size_of::<Unknown>(),
        std::mem::size_of::<JsFunction>()
    );
    unsafe { std::mem::transmute_copy::<Unknown, JsFunction>(&handler) }
}

// ---------------------------------------------------------------------------
// Route handler bridge
// ---------------------------------------------------------------------------

/// A route `Handler` (from the underlying framework) that forwards to a JS callback.
#[derive(Clone)]
struct JsRouteHandler {
    tsfn: Arc<HandlerTsfn>,
}

#[service::async_trait]
impl RouteHandler for JsRouteHandler {
    async fn handle<'r>(&self, req: &'r Request<'_>, data: Data<'r>) -> RouteOutcome<'r> {
        let payload = build_request_payload(req, Some(data)).await;
        match self.tsfn.call_async(Ok((payload,))).await {
            Ok(resp) => RouteOutcome::Success(build_response(resp)),
            Err(_) => RouteOutcome::Error(ServiceStatus::InternalServiceError),
        }
    }
}

// ---------------------------------------------------------------------------
// Catcher handler bridge
// ---------------------------------------------------------------------------

/// A catcher `Handler` (from the underlying framework) that forwards to a JS callback.
#[derive(Clone)]
struct JsCatcherHandler {
    tsfn: Arc<HandlerTsfn>,
}

#[service::async_trait]
impl catcher::Handler for JsCatcherHandler {
    async fn handle<'r>(&self, _status: ServiceStatus, req: &'r Request<'_>) -> catcher::Result<'r> {
        let payload = build_request_payload(req, None).await;
        match self.tsfn.call_async(Ok((payload,))).await {
            Ok(resp) => Ok(build_response(resp)),
            Err(_) => Err(ServiceStatus::InternalServiceError),
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

/// Service configuration, mirroring the underlying framework's `Config` (a `Figment`
/// provider). Any field left unset falls back to the framework's defaults.
#[napi(object)]
pub struct ServiceConfig {
    pub address: Option<String>,
    pub port: Option<u16>,
    pub workers: Option<u32>,
    pub max_blocking: Option<u32>,
    pub keep_alive: Option<u32>,
    pub ident: Option<String>,
    pub log_level: Option<String>,
    pub shutdown: Option<ShutdownOptions>,
}

/// A handle to a service, mirroring the underlying framework's launch entrypoint.
#[napi]
pub struct Service {
    inner: Arc<Mutex<Option<Svc<Build>>>>,
    shutdown: Arc<Mutex<Option<Shutdown>>>,
}

#[napi]
impl Service {
    /// Create a new service instance from the given configuration.
    #[napi(constructor)]
    pub fn new(config: Option<ServiceConfig>) -> Result<Self> {
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
            inner: Arc::new(Mutex::new(Some(service::custom(cfg)))),
            shutdown: Arc::new(Mutex::new(None)),
        })
    }

    /// Mount a single `handler` for `method` at `path` under `base`.
    /// Equivalent to mounting a single `Route` at `base`.
    ///
    /// `handler` is `(err: null, req: RequestPayload) => ResponsePayload | Promise<ResponsePayload>`.
    #[napi]
    #[allow(deprecated)]
    pub fn mount(&self, base: String, method: String, path: String, handler: Unknown) -> Result<()> {
        let method: Method = method
            .parse()
            .map_err(|_| Error::new(Status::GenericFailure, format!("invalid method: {method}")))?;
        let tsfn: HandlerTsfn = to_js_function(handler).create_threadsafe_function(|ctx| Ok(ctx.value))?;
        let route = Route::new(method, &path, JsRouteHandler { tsfn: Arc::new(tsfn) });

        let mut guard = self.inner.lock().unwrap();
        let service = guard
            .take()
            .ok_or_else(|| Error::new(Status::GenericFailure, "service already launched"))?;
        *guard = Some(service.mount(&base, vec![route]));
        Ok(())
    }

    /// Register a single catcher `handler`. `code` is the HTTP status it handles
    /// (in `[400, 600)`); pass `None` for the default catcher.
    /// Equivalent to registering a single `Catcher` at `base`.
    #[napi]
    #[allow(deprecated)]
    pub fn register(&self, base: String, code: Option<u32>, handler: Unknown) -> Result<()> {
        let code: Option<u16> = code.map(|c| c as u16);
        let tsfn: HandlerTsfn = to_js_function(handler).create_threadsafe_function(|ctx| Ok(ctx.value))?;
        let catcher = Catcher::new(code, JsCatcherHandler { tsfn: Arc::new(tsfn) });

        let mut guard = self.inner.lock().unwrap();
        let service = guard
            .take()
            .ok_or_else(|| Error::new(Status::GenericFailure, "service already launched"))?;
        *guard = Some(service.register(&base, vec![catcher]));
        Ok(())
    }

    /// Ready and launch this service, resolving when the service shuts down.
    ///
    /// This mirrors the generated entrypoint from the framework's launch macro:
    #[napi]
    pub async fn launch(&self) -> Result<()> {
        let service = {
            let mut guard = self
                .inner
                .lock()
                .map_err(|_| Error::new(Status::GenericFailure, "lock poisoned"))?;
            guard.take().ok_or_else(|| {
                Error::new(Status::GenericFailure, "service already launched or not configured")
            })?
        };

        let service = service.start().await.map_err(service_err)?;
        let shutdown = service.shutdown();
        *self.shutdown.lock().unwrap() = Some(shutdown);
        service.launch().await.map_err(service_err)?;
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
            None => Err(Error::new(Status::GenericFailure, "service is not running")),
        }
    }
}

/// Convenience mirror of the framework's launch entrypoint: build (with optional
/// config), start, launch, and resolve once the service shuts down.
#[napi]
pub async fn launch(config: Option<ServiceConfig>) -> Result<()> {
    Service::new(config)?.launch().await
}

/// Mirror of the framework's inspectable launch form: start, launch, resolve.
#[napi]
pub async fn service_main(config: Option<ServiceConfig>) -> Result<()> {
    Service::new(config)?.launch().await
}

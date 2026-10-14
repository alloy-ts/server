//! NAPI bindings for the underlying web framework application server.

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

/// Convert any displayable error into an NAPI error.
fn server_err<E: std::fmt::Display>(e: E) -> napi::Error {
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
    /// Raw request body, or `undefined` when the request carries no body.
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

fn extract_route_params(pattern: &str, path: &str) -> HashMap<String, String> {
    let pattern_path = pattern.split('?').next().unwrap_or(pattern);
    let p_segs: Vec<&str> = pattern_path
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
            let is_trailing = inner.ends_with("..>");
            let name = inner
                .strip_suffix("..>")
                .or_else(|| inner.strip_suffix('>'));
            if let Some(name) = name {
                if name != "_" {
                    if is_trailing {
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
    }
    params
}

/// Build a `RequestPayload` from a request. The body is read fully when
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
            extract_route_params(&pattern, &path)
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

type HandlerTsfn = ThreadsafeFunction<(RequestPayload,), ResponsePayload>;

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

#[derive(Clone)]
struct JsRouteHandler {
    tsfn: Arc<HandlerTsfn>,
}

#[rocket::async_trait]
impl RouteHandler for JsRouteHandler {
    async fn handle<'r>(&self, req: &'r Request<'_>, data: Data<'r>) -> RouteOutcome<'r> {
        let payload = build_request_payload(req, Some(data)).await;
        match self.tsfn.call_async(Ok((payload,))).await {
            Ok(resp) => RouteOutcome::Success(build_response(resp)),
            Err(_) => RouteOutcome::Error(RocketStatus::InternalServerError),
        }
    }
}

// ---------------------------------------------------------------------------
// Catcher handler bridge
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct JsCatcherHandler {
    tsfn: Arc<HandlerTsfn>,
}

#[rocket::async_trait]
impl catcher::Handler for JsCatcherHandler {
    async fn handle<'r>(&self, _status: RocketStatus, req: &'r Request<'_>) -> catcher::Result<'r> {
        let payload = build_request_payload(req, None).await;
        match self.tsfn.call_async(Ok((payload,))).await {
            Ok(resp) => Ok(build_response(resp)),
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

/// Server configuration.
/// Any field left unset falls back to the underlying framework's defaults.
#[napi(object)]
pub struct ServerConfig {
    pub address: Option<String>,
    pub port: Option<u16>,
    pub workers: Option<u32>,
    pub max_blocking: Option<u32>,
    pub keep_alive: Option<u32>,
    pub ident: Option<String>,
    pub log_level: Option<String>,
    pub shutdown: Option<ShutdownOptions>,
}

/// A handle to a server instance.
pub struct ServerHandle {
    inner: Arc<Mutex<Option<RocketServer<Build>>>>,
    shutdown: Arc<Mutex<Option<Shutdown>>>,
}

impl ServerHandle {
    pub fn new(config: Option<ServerConfig>) -> Result<Self> {
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

    #[allow(deprecated)]
    pub fn mount(&self, base: String, method: String, path: String, handler: Unknown) -> Result<()> {
        let method: Method = method
            .parse()
            .map_err(|_| Error::new(Status::GenericFailure, format!("invalid method: {method}")))?;
        let tsfn: HandlerTsfn = to_js_function(handler).create_threadsafe_function(|ctx| Ok(ctx.value))?;
        let route = Route::new(method, &path, JsRouteHandler { tsfn: Arc::new(tsfn) });

        let mut guard = self.inner.lock().unwrap();
        let server = guard
            .take()
            .ok_or_else(|| Error::new(Status::GenericFailure, "server already launched"))?;
        *guard = Some(server.mount(&base, vec![route]));
        Ok(())
    }

    #[allow(deprecated)]
    pub fn register(&self, base: String, code: Option<u32>, handler: Unknown) -> Result<()> {
        let code: Option<u16> = code.map(|c| c as u16);
        let tsfn: HandlerTsfn = to_js_function(handler).create_threadsafe_function(|ctx| Ok(ctx.value))?;
        let catcher = Catcher::new(code, JsCatcherHandler { tsfn: Arc::new(tsfn) });

        let mut guard = self.inner.lock().unwrap();
        let server = guard
            .take()
            .ok_or_else(|| Error::new(Status::GenericFailure, "server already launched"))?;
        *guard = Some(server.register(&base, vec![catcher]));
        Ok(())
    }

    pub async fn launch(&self) -> Result<()> {
        let server = {
            let mut guard = self
                .inner
                .lock()
                .map_err(|_| Error::new(Status::GenericFailure, "lock poisoned"))?;
            guard.take().ok_or_else(|| {
                Error::new(Status::GenericFailure, "server already launched or not configured")
            })?
        };

        let server = server.ignite().await.map_err(server_err)?;
        let shutdown = server.shutdown();
        *self.shutdown.lock().unwrap() = Some(shutdown);
        server.launch().await.map_err(server_err)?;
        Ok(())
    }

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

pub async fn launch(config: Option<ServerConfig>) -> Result<()> {
    ServerHandle::new(config)?.launch().await
}

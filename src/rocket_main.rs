// NAPI bindings that let Node.js drive a Rocket server.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeCallContext, ThreadsafeFunction};
#[allow(deprecated)]
use napi::{JsFunction, Unknown};
use napi_derive::napi;

use rocket::catcher::{self, Catcher};
use rocket::config::{Ident, LogLevel, Shutdown as ShutdownConfig};
use rocket::http::{Method, Status as RocketStatus};
use rocket::route::{Handler as RouteHandler, Outcome as RouteOutcome, Route};
use rocket::tokio::io::AsyncReadExt;
use rocket::{Build, Config, Data, Request, Response, Rocket as RocketServer, Shutdown};

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
    /// Raw request body, or `undefined` when the request carries no body.
    pub body: Option<Buffer>,
}

impl Clone for RequestPayload {
    fn clone(&self) -> Self {
        RequestPayload {
            method: self.method.clone(),
            path: self.path.clone(),
            query: self.query.clone(),
            headers: self.headers.clone(),
            params: self.params.clone(),
            body: self.body.as_ref().map(|b| Buffer::from(b.as_ref())),
        }
    }
}

#[derive(Clone)]
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

impl Clone for ResponsePayload {
    fn clone(&self) -> Self {
        ResponsePayload {
            status: self.status,
            headers: self.headers.clone(),
            body: self.body.as_ref().map(|b| Buffer::from(b.as_ref())),
        }
    }
}

/// Build a `RequestPayload` from a Rocket request.
pub(crate) async fn build_request_payload(
    req: &Request<'_>,
    data: Option<Data<'_>>,
) -> RequestPayload {
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
            let mut map = HashMap::new();
            let route_path = route.uri.path().to_string();
            let req_path = req.uri().path().to_string();
            let route_segs: Vec<&str> = route_path.split('/').filter(|s| !s.is_empty()).collect();
            let req_segs: Vec<&str> = req_path.split('/').filter(|s| !s.is_empty()).collect();
            for (i, r_seg) in route_segs.iter().enumerate() {
                if r_seg.starts_with('<') && r_seg.ends_with('>') {
                    let name = r_seg.trim_matches('<').trim_matches('>').trim_end_matches("..");
                    if r_seg.ends_with("..>") {
                        if i < req_segs.len() {
                            let rest = req_segs[i..].join("/");
                            map.insert(name.to_string(), rest);
                        }
                    } else if let Some(&val) = req_segs.get(i) {
                        map.insert(name.to_string(), val.to_string());
                    }
                }
            }
            map
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
        let bytes: Vec<u8> = body.as_ref().to_vec();
        builder.sized_body(bytes.len(), std::io::Cursor::new(bytes));
    }

    builder.finalize()
}

pub(crate) type HandlerTsfn = ThreadsafeFunction<(RequestPayload,), ResponsePayload>;

/// Bridge a JS `Unknown` into a `JsFunction`.
#[allow(deprecated)]
pub(crate) fn to_js_function(handler: Unknown) -> JsFunction {
    debug_assert_eq!(
        std::mem::size_of::<Unknown>(),
        std::mem::size_of::<JsFunction>()
    );
    unsafe { std::mem::transmute_copy::<Unknown, JsFunction>(&handler) }
}

#[allow(deprecated)]
pub(crate) fn wrap_js_handler(env: &Env, val: Unknown) -> Result<JsFunction> {
    let script = r#"
    (fn) => (...args) => {
        function formatRes(res) {
            if (!res) return { status: 200, headers: [], body: null };
            if (typeof res === "string") return { status: 200, headers: [], body: Buffer.from(res) };
            if (Buffer.isBuffer(res) || res instanceof Uint8Array) return { status: 200, headers: [], body: res };

            let status = res.status ?? 200;
            let headers = Array.isArray(res.headers) ? res.headers : [];
            let body = res.body;
            if (body) {
                if (typeof body === "string") {
                    body = Buffer.from(body);
                } else if (Buffer.isBuffer(body) || body instanceof Uint8Array) {
                    body = Buffer.from(body.buffer, body.byteOffset, body.byteLength);
                }
            }
            return { status: status, headers: headers, body: body };
        }

        try {
            let request = null;
            for (const arg of args) {
                if (!arg) continue;
                if (arg.method) { request = arg; break; }
                if (Array.isArray(arg) && arg[0] && arg[0].method) { request = arg[0]; break; }
            }
            let res = fn(request);
            if (res && typeof res.then === "function") {
                return res.then(formatRes).catch(e => ({ status: 500, headers: [], body: Buffer.from(String(e)) }));
            }
            return formatRes(res);
        } catch (e) {
            return { status: 500, headers: [], body: Buffer.from(String(e)) };
        }
    }
    "#;
    let wrap_creator: Unknown = env.run_script(script)?;
    let wrap_creator_fn: JsFunction = unsafe { std::mem::transmute_copy(&wrap_creator) };
    let js_fn = to_js_function(val);
    let wrapped: Unknown = wrap_creator_fn.call(None, &[&js_fn])?;
    let wrapped_fn: JsFunction = unsafe { std::mem::transmute_copy(&wrapped) };
    Ok(wrapped_fn)
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

#[napi]
pub struct Rocket {
    inner: Arc<Mutex<Option<RocketServer<Build>>>>,
    shutdown: Arc<Mutex<Option<Shutdown>>>,
}

#[napi]
impl Rocket {
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

    #[napi]
    #[allow(deprecated)]
    pub fn mount(&self, env: Env, base: String, method: String, path: String, handler: Unknown) -> Result<()> {
        let method: Method = method
            .parse()
            .map_err(|_| Error::new(Status::GenericFailure, format!("invalid method: {method}")))?;
        let wrapped_fn = wrap_js_handler(&env, handler)?;
        let tsfn: HandlerTsfn = wrapped_fn.create_threadsafe_function(|ctx: ThreadsafeCallContext<(RequestPayload,)>| Ok(ctx.value))?;
        let route = Route::new(method, &path, JsRouteHandler { tsfn: Arc::new(tsfn) });

        let mut guard = self.inner.lock().unwrap();
        let rocket = guard
            .take()
            .ok_or_else(|| Error::new(Status::GenericFailure, "server already launched"))?;
        *guard = Some(rocket.mount(&base, vec![route]));
        Ok(())
    }

    #[napi]
    #[allow(deprecated)]
    pub fn register(&self, env: Env, base: String, code: Option<u32>, handler: Unknown) -> Result<()> {
        let code: Option<u16> = code.map(|c| c as u16);
        let wrapped_fn = wrap_js_handler(&env, handler)?;
        let tsfn: HandlerTsfn = wrapped_fn.create_threadsafe_function(|ctx: ThreadsafeCallContext<(RequestPayload,)>| Ok(ctx.value))?;
        let catcher = Catcher::new(code, JsCatcherHandler { tsfn: Arc::new(tsfn) });

        let mut guard = self.inner.lock().unwrap();
        let rocket = guard
            .take()
            .ok_or_else(|| Error::new(Status::GenericFailure, "server already launched"))?;
        *guard = Some(rocket.register(&base, vec![catcher]));
        Ok(())
    }

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

#[napi]
pub async fn launch(config: Option<RocketConfig>) -> Result<()> {
    Rocket::new(config)?.launch().await
}

#[napi]
pub async fn rocket_main(config: Option<RocketConfig>) -> Result<()> {
    Rocket::new(config)?.launch().await
}

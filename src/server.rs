// High-level, Bun-style HTTP server exposed to JavaScript via `#[napi]`.

use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Arc, Mutex};

use napi::bindgen_prelude::*;
use napi_derive::napi;

use rocket::catcher::{self, Catcher};
use rocket::http::{Method, Status as RocketStatus};
use rocket::route::{Handler as RouteHandler, Outcome as RouteOutcome, Route};
use rocket::{Config, Data, Request, Shutdown};

use crate::rocket_main::{
    build_request_payload, build_response, wrap_js_handler, HandlerTsfn, JsHeader,
    ResponsePayload,
};

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
pub fn translate_pattern(pattern: &str) -> String {
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

// ---------------------------------------------------------------------------
// `Response` (public class) + `ResponseInit` + `FileRef`
// ---------------------------------------------------------------------------

/// Options for the `Response` constructor / `Response.json`.
#[napi(object)]
pub struct ResponseInit {
    pub status: Option<u32>,
    pub headers: Option<Vec<JsHeader>>,
}

/// Alloy's `Response`, mirroring the Bun/Web shape.
#[napi]
pub struct Response {
    pub status: u32,
    pub(crate) headers: Vec<JsHeader>,
    pub(crate) body: Option<Buffer>,
}

#[napi]
impl Response {
    #[napi(constructor)]
    pub fn new(body: Option<String>, init: Option<ResponseInit>) -> Self {
        let status = init.as_ref().and_then(|i| i.status).unwrap_or(200);
        let headers = init.and_then(|i| i.headers.clone()).unwrap_or_default();
        let body = body.map(|s| Buffer::from(s.into_bytes()));
        Response {
            status,
            headers,
            body,
        }
    }

    #[napi(getter)]
    pub fn headers(&self) -> Vec<JsHeader> {
        self.headers.clone()
    }

    #[napi(getter)]
    pub fn body(&self) -> Option<Buffer> {
        self.body.as_ref().map(|b| Buffer::from(b.as_ref()))
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
            status: init.as_ref().and_then(|i| i.status).unwrap_or(200),
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

/// `file(path)` — lazy static file serving.
#[napi]
pub fn file(path: String) -> FileRef {
    FileRef { path }
}

// ---------------------------------------------------------------------------
// Route handler bridge
// ---------------------------------------------------------------------------

enum RouteKind {
    Fn(Arc<HandlerTsfn>),
    Static(ResponsePayload),
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
        let has_body = match req.method() {
            Method::Get | Method::Head | Method::Options => false,
            _ => true,
        };
        match &*self.kind {
            RouteKind::Fn(tsfn) => {
                let payload = build_request_payload(req, if has_body { Some(data) } else { None }).await;
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
    #[napi(getter)]
    pub fn port(&self) -> u16 {
        self.port
    }

    #[napi(getter)]
    pub fn url(&self) -> String {
        match &self.unix {
            Some(path) => format!("unix://{path}"),
            None => format!("http://{}:{}", self.host, self.port),
        }
    }

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

fn push_kind(kind: RouteKind, pattern: &str, methods: &[Method], out: &mut Vec<Route>) {
    let handler = ServeHandler {
        kind: Arc::new(kind),
    };
    let rocket_pattern = translate_pattern(pattern);
    for m in methods {
        out.push(Route::new(*m, &rocket_pattern, handler.clone()));
    }
}

fn mount_value(env: &Env, value: Unknown, pattern: &str, methods: &[Method], out: &mut Vec<Route>) -> Result<()> {
    match value.get_type()? {
        ValueType::Function => {
            let wrapped_fn = wrap_js_handler(env, value)?;
            let tsfn: HandlerTsfn = wrapped_fn.create_threadsafe_function(|ctx| Ok(ctx.value))?;
            push_kind(RouteKind::Fn(Arc::new(tsfn)), pattern, methods, out);
        }
        ValueType::Object => {
            let obj: Object = unsafe { value.cast()? };
            if obj.has_named_property("path")? {
                let path: String = obj.get_named_property("path")?;
                push_kind(
                    RouteKind::File {
                        path,
                        cache: Arc::new(Mutex::new(None)),
                    },
                    pattern,
                    methods,
                    out,
                );
            } else if obj.has_named_property("status")?
                || obj.has_named_property("headers")?
                || obj.has_named_property("body")?
            {
                let status: Option<u32> = obj.get_named_property("status")?;
                let headers: Option<Vec<JsHeader>> = obj.get_named_property("headers")?;
                let body: Option<Buffer> = obj.get_named_property("body")?;
                push_kind(
                    RouteKind::Static(ResponsePayload {
                        status,
                        headers,
                        body,
                    }),
                    pattern,
                    methods,
                    out,
                );
            } else {
                let keys = obj.get_property_names()?;
                let len = keys.get_array_length()?;
                for i in 0..len {
                    let method_str: String = keys.get_element(i)?;
                    let method_val: Unknown = obj.get_named_property(&method_str)?;
                    let m: Method = method_str
                        .parse()
                        .map_err(|_| Error::new(Status::GenericFailure, format!("invalid method: {method_str}")))?;
                    mount_value(env, method_val, pattern, &[m], out)?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

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

#[napi]
pub fn serve(env: Env, options: Object) -> Result<Server> {
    let port: Option<u32> = options.get_named_property("port").ok().flatten();
    let hostname: Option<String> = options.get_named_property("hostname").ok().flatten();
    let unix: Option<String> = options.get_named_property("unix").ok().flatten();
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

    let mut cfg = Config::default();
    cfg.port = port;
    let ip: IpAddr = match hostname.parse::<IpAddr>() {
        Ok(ip) => ip,
        Err(_) if hostname == "localhost" => IpAddr::V4(Ipv4Addr::LOCALHOST),
        Err(_) => IpAddr::V4(Ipv4Addr::UNSPECIFIED),
    };
    cfg.address = ip;
    let _ = is_unix;

    let mut rocket = rocket::custom(cfg);

    let mut routes: Vec<Route> = Vec::new();
    if let Ok(Some(table)) = options.get_named_property::<Option<Object>>("routes") {
        let keys = table.get_property_names()?;
        let len = keys.get_array_length()?;
        for i in 0..len {
            let pattern: String = keys.get_element(i)?;
            let value: Unknown = table.get_named_property(&pattern)?;
            mount_value(&env, value, &pattern, ALL_METHODS, &mut routes)?;
        }
    }
    if !routes.is_empty() {
        rocket = rocket.mount("/", routes);
    }

    let fetch_prop: Option<Unknown> = options.get_named_property("fetch").ok().flatten();
    let fetch_tsfn: Option<Arc<HandlerTsfn>> = match fetch_prop {
        Some(val) if val.get_type()? == ValueType::Function => {
            let wrapped_fn = wrap_js_handler(&env, val)?;
            Some(Arc::new(wrapped_fn.create_threadsafe_function(|ctx| Ok(ctx.value))?))
        }
        _ => None,
    };
    rocket = rocket.register("/", vec![Catcher::new(Some(404), ServeCatcher { tsfn: fetch_tsfn })]);

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

//! # NAPI bindings that bridge Rocket's runtime to TypeScript handlers.

use std::collections::HashMap;
use std::io::Cursor;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Arc, Mutex};

use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
#[allow(deprecated)]
use napi::{JsFunction, Unknown, ValueType};
use napi_derive::napi;

use rocket::config::Config;
use rocket::data::ByteUnit;
use rocket::http::{Header, Method, Status as RocketStatus};
use rocket::request::Request;
use rocket::response::Response as RocketResponse;
use rocket::route::{Handler, Outcome as RouteOutcome, Route};
use rocket::{Data, Shutdown};

type HandlerTsfn =
    ThreadsafeFunction<RequestInput, Promise<ResponsePayload>, RequestInput, Status, true, false, 0>;

#[napi(object)]
pub struct RequestInput {
    pub method: String,
    pub path: String,
    pub query: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Buffer>,
    pub params: HashMap<String, String>,
}

#[napi(object)]
pub struct ResponseInit {
    pub status: Option<u32>,
    pub headers: Option<Vec<(String, String)>>,
}

#[napi(object)]
pub struct ResponsePayload {
    pub status: u32,
    pub headers: Vec<(String, String)>,
    pub body: Option<Buffer>,
}

#[napi]
pub struct Response {
    status_code: u32,
    headers_list: Vec<(String, String)>,
    body_buf: Option<Vec<u8>>,
}

#[napi]
impl Response {
    #[napi(constructor)]
    pub fn new(body: Option<String>, init: Option<ResponseInit>) -> Self {
        let init = init.unwrap_or(ResponseInit {
            status: None,
            headers: None,
        });
        let body_buf = body.map(|s| s.into_bytes());
        Response {
            status_code: init.status.unwrap_or(200),
            headers_list: init.headers.unwrap_or_default(),
            body_buf,
        }
    }

    #[napi(getter)]
    pub fn status(&self) -> u32 {
        self.status_code
    }

    #[napi(getter)]
    pub fn headers(&self) -> Vec<(String, String)> {
        self.headers_list.clone()
    }

    #[napi(getter)]
    pub fn body(&self) -> Option<Buffer> {
        self.body_buf.as_ref().map(|b| Buffer::from(b.as_slice()))
    }

    #[napi]
    pub fn json(data: serde_json::Value, init: Option<ResponseInit>) -> napi::Result<Self> {
        let init = init.unwrap_or(ResponseInit {
            status: None,
            headers: None,
        });
        let bytes = serde_json::to_vec(&data)
            .map_err(|e| napi::Error::new(Status::GenericFailure, e.to_string()))?;
        let mut headers = init.headers.unwrap_or_default();
        headers.push((
            "Content-Type".to_string(),
            "application/json".to_string(),
        ));
        Ok(Response {
            status_code: init.status.unwrap_or(200),
            headers_list: headers,
            body_buf: Some(bytes),
        })
    }

    #[napi]
    pub fn redirect(url: String, status: Option<u32>) -> Self {
        Response {
            status_code: status.unwrap_or(302),
            headers_list: vec![("Location".to_string(), url)],
            body_buf: None,
        }
    }
}

/// Helper function to build a Response object.
#[napi]
pub fn response(
    body: Option<String>,
    init_or_status: Option<Unknown>,
    headers: Option<Vec<(String, String)>>,
) -> Response {
    if let Some(val) = init_or_status {
        if val.get_type().ok() == Some(ValueType::Number) {
            let status: u32 = unsafe { val.cast().unwrap_or(200) };
            return Response::new(
                body,
                Some(ResponseInit {
                    status: Some(status),
                    headers,
                }),
            );
        } else if val.get_type().ok() == Some(ValueType::Object) {
            let obj: Object = unsafe { val.cast().unwrap() };
            let status: Option<u32> = obj.get_named_property("status").ok();
            let hdrs: Option<Vec<(String, String)>> = obj.get_named_property("headers").ok();
            return Response::new(
                body,
                Some(ResponseInit {
                    status,
                    headers: hdrs.or(headers),
                }),
            );
        }
    }
    Response::new(
        body,
        Some(ResponseInit {
            status: None,
            headers,
        }),
    )
}

#[napi(object)]
pub struct StaticFile {
    pub path: String,
}

#[napi]
pub fn file(path: String) -> StaticFile {
    StaticFile { path }
}

#[derive(Clone)]
enum RouteKind {
    Fn(Arc<HandlerTsfn>),
    Static(Arc<ResponsePayload>),
    File { path: String },
}

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
            RouteKind::File { path } => {
                match rocket::tokio::fs::read(path).await {
                    Ok(bytes) => {
                        let mut builder = RocketResponse::build();
                        builder.sized_body(bytes.len(), Cursor::new(bytes));
                        RouteOutcome::Success(builder.finalize())
                    }
                    Err(_) => RouteOutcome::Error(RocketStatus::NotFound),
                }
            }
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
        .map(|h| (h.name().as_str().to_string(), h.value().to_string()))
        .collect();

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
    if !out.starts_with('/') {
        out.insert(0, '/');
    }
    out
}

fn build_response(payload: &ResponsePayload) -> RocketResponse<'static> {
    let mut builder = RocketResponse::build();
    if let Some(status) = u16::try_from(payload.status).ok().and_then(RocketStatus::from_code) {
        builder.status(status);
    }
    for (name, value) in &payload.headers {
        builder.header(Header::new(name.clone(), value.clone()));
    }
    if let Some(body) = &payload.body {
        let bytes: Vec<u8> = body.as_ref().to_vec();
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
fn to_js_function(handler: Unknown) -> napi::Result<JsFunction> {
    if handler.get_type()? == ValueType::Function {
        Ok(unsafe { std::mem::transmute_copy::<Unknown, JsFunction>(&handler) })
    } else {
        Err(napi::Error::new(
            Status::InvalidArg,
            "expected a function handler",
        ))
    }
}

fn mount_value(value: Unknown, pattern: &str, methods: &[Method], out: &mut Vec<Route>) -> napi::Result<()> {
    match value.get_type()? {
        ValueType::Function => {
            let f = to_js_function(value)?;
            let tsfn: HandlerTsfn = f.create_threadsafe_function(|ctx| Ok(ctx.value))?;
            push_kind(RouteKind::Fn(Arc::new(tsfn)), pattern, methods, out);
        }
        ValueType::Object => {
            let obj: Object = unsafe { value.clone().cast()? };
            if obj.has_named_property("path")? {
                if let Some(path) = obj.get_named_property::<Option<String>>("path")? {
                    push_kind(RouteKind::File { path }, pattern, methods, out);
                    return Ok(());
                }
            }
            if obj.has_named_property("status")? {
                let status: u32 = obj.get_named_property("status").unwrap_or(200);
                let headers: Vec<(String, String)> = obj.get_named_property("headers").unwrap_or_default();
                let body: Option<Buffer> = obj.get_named_property("body").unwrap_or(None);
                push_kind(
                    RouteKind::Static(Arc::new(ResponsePayload {
                        status,
                        headers,
                        body,
                    })),
                    pattern,
                    methods,
                    out,
                );
                return Ok(());
            }

            let names: Object = obj.get_property_names()?;
            let len = names.get_array_length()?;
            for i in 0..len {
                let method_name: String = names.get_element(i)?;
                if let Ok(m) = method_name.parse::<Method>() {
                    let method_value: Unknown = obj.get_named_property(&method_name)?;
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

#[napi]
pub fn serve(options: Option<Unknown>) -> napi::Result<Server> {
    let env_port = std::env::var("BUN_PORT")
        .or_else(|_| std::env::var("PORT"))
        .or_else(|_| std::env::var("NODE_PORT"))
        .ok()
        .and_then(|s| s.parse::<u16>().ok());

    let mut port: u16 = env_port.unwrap_or(3000);
    let mut hostname = "0.0.0.0".to_string();

    let mut routes: Vec<Route> = Vec::new();

    if let Some(options) = options {
        if let ValueType::Object = options.get_type()? {
            let obj: Object = unsafe { options.cast()? };

            if obj.has_named_property("port")? {
                if let Some(p) = obj.get_named_property::<Option<u32>>("port")? {
                    port = p as u16;
                }
            }
            if obj.has_named_property("hostname")? {
                if let Some(h) = obj.get_named_property::<Option<String>>("hostname")? {
                    hostname = h;
                }
            }

            if obj.has_named_property("routes")? {
                if let Some(table) = obj.get_named_property::<Option<Object>>("routes")? {
                    let names: Object = table.get_property_names()?;
                    let len = names.get_array_length()?;
                    for i in 0..len {
                        let pattern: String = names.get_element(i)?;
                        let value: Unknown = table.get_named_property(&pattern)?;
                        mount_value(value, &pattern, ALL_METHODS, &mut routes)?;
                    }
                }
            }

            if obj.has_named_property("fetch")? {
                if let Some(fetch) = obj.get_named_property::<Option<Unknown>>("fetch")? {
                    if fetch.get_type()? == ValueType::Function {
                        let f = to_js_function(fetch)?;
                        let tsfn: HandlerTsfn = f.create_threadsafe_function(|ctx| Ok(ctx.value))?;
                        let tsfn_arc = Arc::new(tsfn);
                        for m in ALL_METHODS {
                            push_kind(
                                RouteKind::Fn(tsfn_arc.clone()),
                                "/<_..>",
                                &[*m],
                                &mut routes,
                            );
                        }
                    }
                }
            }
        }
    }

    let address: IpAddr = match hostname.as_str() {
        "localhost" => IpAddr::V4(Ipv4Addr::LOCALHOST),
        other => other.parse().unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
    };

    let mut config = Config::default();
    config.port = port;
    config.address = address;

    let rocket = rocket::build().configure(config).mount("/", routes);

    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let rt = rocket::tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
            match rocket.ignite().await {
                Ok(ignited) => {
                    let bound_port = ignited.config().port;
                    let shutdown = ignited.shutdown();
                    let _ = tx.send(Ok((bound_port, shutdown)));
                    let _ = ignited.launch().await;
                }
                Err(e) => {
                    let _ = tx.send(Err(e.to_string()));
                }
            }
        });
    });

    let (bound_port, shutdown) = rx
        .recv()
        .map_err(|_| napi::Error::new(Status::GenericFailure, "Failed to start Rocket server"))?
        .map_err(|e| napi::Error::new(Status::GenericFailure, e))?;

    let url = format!("http://{}:{}", hostname, bound_port);

    Ok(Server {
        url,
        port: bound_port as u32,
        shutdown: Arc::new(Mutex::new(Some(shutdown))),
    })
}

#[napi]
pub struct Server {
    pub url: String,
    pub port: u32,
    shutdown: Arc<Mutex<Option<Shutdown>>>,
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

/// App class re-expressing rocket::build() route registration in TypeScript.
#[napi]
pub struct App {
    routes: Vec<Route>,
    active_server: Arc<Mutex<Option<Server>>>,
}

#[napi]
impl App {
    #[napi(constructor)]
    pub fn new() -> Self {
        App {
            routes: Vec::new(),
            active_server: Arc::new(Mutex::new(None)),
        }
    }

    #[napi]
    pub fn get(&mut self, path: String, handler: Unknown) -> napi::Result<()> {
        mount_value(handler, &path, &[Method::Get], &mut self.routes)
    }

    #[napi]
    pub fn post(&mut self, path: String, handler: Unknown) -> napi::Result<()> {
        mount_value(handler, &path, &[Method::Post], &mut self.routes)
    }

    #[napi]
    pub fn put(&mut self, path: String, handler: Unknown) -> napi::Result<()> {
        mount_value(handler, &path, &[Method::Put], &mut self.routes)
    }

    #[napi]
    pub fn delete(&mut self, path: String, handler: Unknown) -> napi::Result<()> {
        mount_value(handler, &path, &[Method::Delete], &mut self.routes)
    }

    #[napi]
    pub fn patch(&mut self, path: String, handler: Unknown) -> napi::Result<()> {
        mount_value(handler, &path, &[Method::Patch], &mut self.routes)
    }

    #[napi]
    pub fn mount(&mut self, base: String, routes: Unknown) -> napi::Result<()> {
        if routes.get_type()? == ValueType::Object {
            let obj: Object = unsafe { routes.cast()? };
            let names = obj.get_property_names()?;
            let len = names.get_array_length()?;
            let prefix = base.trim_end_matches('/');
            for i in 0..len {
                let p: String = names.get_element(i)?;
                let full_path = if p.starts_with('/') {
                    format!("{prefix}{p}")
                } else {
                    format!("{prefix}/{p}")
                };
                let h: Unknown = obj.get_named_property(&p)?;
                mount_value(h, &full_path, ALL_METHODS, &mut self.routes)?;
            }
        }
        Ok(())
    }

    #[napi]
    pub fn launch(&mut self, options: Option<Unknown>) -> napi::Result<Server> {
        let mut port: u16 = 3000;
        let mut hostname = "0.0.0.0".to_string();

        if let Some(user_opts) = options {
            if user_opts.get_type()? == ValueType::Object {
                let obj: Object = unsafe { user_opts.cast()? };
                if obj.has_named_property("port")? {
                    if let Some(p) = obj.get_named_property::<Option<u32>>("port")? {
                        port = p as u16;
                    }
                }
                if obj.has_named_property("hostname")? {
                    if let Some(h) = obj.get_named_property::<Option<String>>("hostname")? {
                        hostname = h;
                    }
                }
            }
        }

        let address: IpAddr = match hostname.as_str() {
            "localhost" => IpAddr::V4(Ipv4Addr::LOCALHOST),
            other => other.parse().unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
        };

        let mut config = Config::default();
        config.port = port;
        config.address = address;

        let routes = std::mem::take(&mut self.routes);
        let rocket = rocket::build().configure(config).mount("/", routes);

        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let rt = rocket::tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async move {
                match rocket.ignite().await {
                    Ok(ignited) => {
                        let bound_port = ignited.config().port;
                        let shutdown = ignited.shutdown();
                        let _ = tx.send(Ok((bound_port, shutdown)));
                        let _ = ignited.launch().await;
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e.to_string()));
                    }
                }
            });
        });

        let (bound_port, shutdown) = rx
            .recv()
            .map_err(|_| napi::Error::new(Status::GenericFailure, "Failed to start Rocket server"))?
            .map_err(|e| napi::Error::new(Status::GenericFailure, e))?;

        let url = format!("http://{}:{}", hostname, bound_port);

        let server = Server {
            url,
            port: bound_port as u32,
            shutdown: Arc::new(Mutex::new(Some(shutdown))),
        };

        *self.active_server.lock().unwrap() = Some(Server {
            url: server.url.clone(),
            port: server.port,
            shutdown: server.shutdown.clone(),
        });

        Ok(server)
    }

    #[napi]
    pub fn stop(&mut self) -> napi::Result<()> {
        if let Some(s) = self.active_server.lock().unwrap().take() {
            s.stop()?;
        }
        Ok(())
    }
}

/// Namespace providing Alloy server utilities.
#[napi]
pub struct Alloy;

#[napi]
impl Alloy {
    #[napi]
    pub fn serve(options: Option<Unknown>) -> napi::Result<Server> {
        serve(options)
    }

    #[napi]
    pub fn file(path: String) -> StaticFile {
        file(path)
    }

    #[napi]
    pub fn response(
        body: Option<String>,
        init_or_status: Option<Unknown>,
        headers: Option<Vec<(String, String)>>,
    ) -> Response {
        response(body, init_or_status, headers)
    }

    #[napi]
    pub fn app() -> App {
        App::new()
    }
}

static ROUTES: std::sync::OnceLock<Mutex<Vec<Route>>> = std::sync::OnceLock::new();

fn global_routes() -> &'static Mutex<Vec<Route>> {
    ROUTES.get_or_init(|| Mutex::new(Vec::new()))
}

#[napi]
pub fn register_route(method: String, path: String, handler: Unknown) -> napi::Result<()> {
    if handler.get_type()? == ValueType::Function {
        let f = to_js_function(handler)?;
        let tsfn: HandlerTsfn = f.create_threadsafe_function(|ctx| Ok(ctx.value))?;
        let m: Method = method
            .parse()
            .map_err(|_| napi::Error::new(Status::GenericFailure, format!("invalid method: {method}")))?;
        let mut routes = global_routes().lock().unwrap();
        push_kind(RouteKind::Fn(Arc::new(tsfn)), &path, &[m], &mut routes);
    }
    Ok(())
}

#[napi]
pub async fn launch() -> napi::Result<()> {
    let routes = std::mem::take(&mut *global_routes().lock().unwrap());
    rocket::build()
        .mount("/", routes)
        .launch()
        .await
        .map(|_orbit| ())
        .map_err(|e| napi::Error::new(Status::GenericFailure, e.to_string()))
}

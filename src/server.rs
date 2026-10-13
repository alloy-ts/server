//! # NAPI bindings that bridge Rocket's runtime to TypeScript handlers.

use std::collections::HashMap;
use std::io::Cursor;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Arc, Mutex, OnceLock};

use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
#[allow(deprecated)]
use napi::JsFunction;
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

#[allow(deprecated)]
fn to_js_function(val: Unknown) -> JsFunction {
    unsafe { std::mem::transmute_copy::<Unknown, JsFunction>(&val) }
}

/// Wire type sent from Rust -> JS for every request.
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
pub struct ResponseInit {
    pub status: Option<u32>,
    pub headers: Option<Vec<(String, String)>>,
}

/// The `Response` value a JS handler returns.
#[napi(object)]
pub struct ResponsePayload {
    pub status: u32,
    pub headers: Vec<(String, String)>,
    pub body: Option<Buffer>,
}

/// The `Response` class constructed in JS.
#[napi]
pub struct Response {
    status: u32,
    headers: Vec<(String, String)>,
    body: Option<Vec<u8>>,
}

#[napi]
impl Response {
    #[napi(constructor)]
    pub fn new(body: Option<String>, init: Option<ResponseInit>) -> Self {
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
            status: init.status.unwrap_or(200),
            headers,
            body: Some(bytes),
        })
    }

    #[napi]
    pub fn redirect(url: String, status: Option<u32>) -> Self {
        Response {
            status: status.unwrap_or(302),
            headers: vec![("Location".to_string(), url)],
            body: None,
        }
    }

    #[napi(getter)]
    pub fn status(&self) -> u32 {
        self.status
    }

    #[napi(getter)]
    pub fn headers(&self) -> Vec<(String, String)> {
        self.headers.clone()
    }

    #[napi(getter)]
    pub fn body(&self) -> Option<Buffer> {
        self.body.as_ref().map(|b| Buffer::from(b.clone()))
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
enum RouteKind {
    Fn(Arc<HandlerTsfn>),
    Static(Arc<ResponsePayload>),
    File {
        path: String,
        _cache: Arc<Mutex<Option<Vec<u8>>>>,
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
            RouteKind::File { path, _cache: _ } => {
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

/// Extract request data.
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

fn mount_value(value: Unknown, pattern: &str, methods: &[Method], out: &mut Vec<Route>) -> napi::Result<()> {
    match value.get_type()? {
        ValueType::Function => {
            let f = to_js_function(value);
            let tsfn: HandlerTsfn = f.create_threadsafe_function(|ctx| Ok(ctx.value))?;
            push_kind(RouteKind::Fn(Arc::new(tsfn)), pattern, methods, out);
        }
        ValueType::Object => {
            let obj: Object = unsafe { std::mem::transmute_copy(&value) };
            if obj.has_named_property("path")? {
                let path: String = obj.get_named_property("path")?;
                push_kind(
                    RouteKind::File {
                        path,
                        _cache: Arc::new(Mutex::new(None)),
                    },
                    pattern,
                    methods,
                    out,
                );
            } else if obj.has_named_property("status")? {
                let status: u32 = obj.get_named_property("status")?;
                let headers: Vec<(String, String)> = obj.get_named_property("headers").unwrap_or_default();
                let body: Option<Buffer> = obj.get_named_property("body").ok().flatten();
                push_kind(
                    RouteKind::Static(Arc::new(ResponsePayload { status, headers, body })),
                    pattern,
                    methods,
                    out,
                );
            } else {
                let names = obj.get_property_names()?;
                let len = names.get_array_length()?;
                for i in 0..len {
                    let method_str: String = names.get_element(i)?;
                    let method_val: Unknown = obj.get_named_property(&method_str)?;
                    let m: Method = method_str
                        .parse()
                        .map_err(|_| napi::Error::new(Status::GenericFailure, format!("invalid method: {method_str}")))?;
                    mount_value(method_val, pattern, &[m], out)?;
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

static SHUTDOWN: OnceLock<Mutex<Option<Shutdown>>> = OnceLock::new();

/// NAPI `serve(options)` function.
#[napi]
pub fn serve(options: Unknown) -> napi::Result<Server> {
    let opts: Object = unsafe { std::mem::transmute_copy(&options) };
    let port: u16 = opts.get_named_property::<Option<u32>>("port")?.unwrap_or(3000) as u16;
    let hostname: String = opts
        .get_named_property::<Option<String>>("hostname")?
        .unwrap_or_else(|| "0.0.0.0".to_string());
    let address: IpAddr = match hostname.as_str() {
        "localhost" => IpAddr::V4(Ipv4Addr::LOCALHOST),
        other => other.parse().unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
    };
    let url = format!("http://{}:{}", hostname, port);

    let mut routes: Vec<Route> = Vec::new();
    if let Ok(Some(table)) = opts.get_named_property::<Option<Object>>("routes") {
        let names = table.get_property_names()?;
        let len = names.get_array_length()?;
        for i in 0..len {
            let pattern: String = names.get_element(i)?;
            let value: Unknown = table.get_named_property(&pattern)?;
            mount_value(value, &pattern, ALL_METHODS, &mut routes)?;
        }
    }

    if let Ok(Some(fetch_val)) = opts.get_named_property::<Option<Unknown>>("fetch") {
        let fetch_fn = to_js_function(fetch_val);
        let tsfn = Arc::new(fetch_fn.create_threadsafe_function(|ctx| Ok(ctx.value))?);
        for m in ALL_METHODS {
            push_kind(
                RouteKind::Fn(tsfn.clone()),
                "<_..>",
                &[*m],
                &mut routes,
            );
        }
    }

    let mut config = Config::default();
    config.port = port;
    config.address = address;

    let rocket = rocket::build().configure(config).mount("/", routes);

    rocket::tokio::spawn(async move {
        if let Ok(ignited) = rocket.ignite().await {
            let shutdown = ignited.shutdown();
            *SHUTDOWN.get_or_init(|| Mutex::new(None)).lock().unwrap() = Some(shutdown);
            let _ = ignited.launch().await;
        }
    });

    Ok(Server {
        url,
        port: port as u32,
    })
}

/// Handle returned by [`serve`].
#[napi]
pub struct Server {
    pub url: String,
    pub port: u32,
}

#[napi]
impl Server {
    #[napi]
    pub fn stop(&self) -> napi::Result<()> {
        if let Some(lock) = SHUTDOWN.get() {
            if let Some(shutdown) = lock.lock().unwrap().take() {
                shutdown.notify();
            }
        }
        Ok(())
    }
}

static ROUTES: OnceLock<Mutex<Vec<Route>>> = OnceLock::new();

fn routes() -> &'static Mutex<Vec<Route>> {
    ROUTES.get_or_init(|| Mutex::new(Vec::new()))
}

#[napi]
pub fn register_route(method: String, path: String, handler: Unknown) -> napi::Result<()> {
    let f = to_js_function(handler);
    let tsfn: HandlerTsfn = f.create_threadsafe_function(|ctx| Ok(ctx.value))?;
    let m: Method = method
        .parse()
        .map_err(|_| napi::Error::new(Status::GenericFailure, format!("invalid method: {method}")))?;
    let mut routes = routes().lock().unwrap();
    push_kind(RouteKind::Fn(Arc::new(tsfn)), &path, &[m], &mut routes);
    Ok(())
}

#[napi]
pub async fn launch() -> napi::Result<()> {
    let routes = std::mem::take(&mut *routes().lock().unwrap());
    rocket::build()
        .mount("/", routes)
        .launch()
        .await
        .map(|_orbit| ())
        .map_err(|e| napi::Error::new(Status::GenericFailure, e.to_string()))
}

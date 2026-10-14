//! # NAPI bindings that bridge Rocket's runtime to TypeScript handlers.

use std::collections::HashMap;
use std::io::Cursor;
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use std::sync::{Arc, Mutex, OnceLock};

use napi::bindgen_prelude::{Buffer, Function, Promise, Status, Unknown};
use napi::threadsafe_function::ThreadsafeFunction;
#[allow(deprecated)]
use napi::JsObject;
use napi::ValueType;
use napi_derive::napi;

use rocket::data::ByteUnit;
use rocket::http::{Header, Method, Status as RocketStatus};
use rocket::request::Request;
use rocket::response::Response as RocketResponse;
use rocket::route::{Handler, Outcome, Route as RocketRouteHandle};
use rocket::{Data, Shutdown as RocketShutdownHandle};

// ---------------------------------------------------------------------------
// Wire Types & Response
// ---------------------------------------------------------------------------

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
#[derive(Clone)]
pub struct ResponseInit {
    pub status: Option<u32>,
    pub headers: Option<HashMap<String, String>>,
}

#[napi]
pub struct Response {
    status: u32,
    headers: HashMap<String, String>,
    body: Option<Buffer>,
}

#[napi]
impl Response {
    #[napi(getter)]
    pub fn status(&self) -> u32 {
        self.status
    }

    #[napi(getter)]
    pub fn headers(&self) -> HashMap<String, String> {
        self.headers.clone()
    }

    #[napi(getter)]
    pub fn body(&self) -> Option<Buffer> {
        self.body.as_ref().map(|b| Buffer::from(b.as_ref().to_vec()))
    }

    #[napi(constructor)]
    pub fn new(body: Option<Unknown>, init: Option<ResponseInit>) -> napi::Result<Self> {
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

impl Response {
    pub fn to_payload(&self) -> ResponsePayload {
        ResponsePayload {
            status: Some(self.status),
            headers: Some(self.headers.clone()),
            body: self.body.as_ref().map(|b| Buffer::from(b.as_ref().to_vec())),
        }
    }
}

#[napi(object)]
pub struct ResponsePayload {
    pub status: Option<u32>,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<Buffer>,
}

impl Clone for ResponsePayload {
    fn clone(&self) -> Self {
        Self {
            status: self.status,
            headers: self.headers.clone(),
            body: self.body.as_ref().map(|b| Buffer::from(b.as_ref().to_vec())),
        }
    }
}

#[napi(object)]
#[derive(Clone)]
pub struct StaticFile {
    pub path: String,
}

#[napi]
pub fn file(path: String) -> StaticFile {
    StaticFile { path }
}

type RouteHandlerFn =
    ThreadsafeFunction<RequestInput, Promise<ResponsePayload>, RequestInput, Status, false, false, 0>;

#[derive(Clone)]
enum RouteHandler {
    Dynamic(Arc<RouteHandlerFn>),
    Static(ResponsePayload),
    File(String),
}

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
                let output = match tsfn.call_async(input).await {
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
            out.push_str("<path..>");
            i += 1;
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    if out.is_empty() || !out.starts_with('/') {
        out.insert(0, '/');
    }
    out
}

fn build_response(out: ResponsePayload) -> RocketResponse<'static> {
    let mut builder = RocketResponse::build();
    if let Some(status) = RocketStatus::from_code(out.status.unwrap_or(200) as u16) {
        builder.status(status);
    }
    for (name, value) in out.headers.unwrap_or_default() {
        builder.header(Header::new(name, value));
    }
    if let Some(body) = out.body {
        let bytes: Vec<u8> = body.as_ref().to_vec();
        builder.sized_body(bytes.len(), Cursor::new(bytes));
    }
    builder.finalize()
}

#[napi(object)]
pub struct ServeOptions {
    pub port: Option<u32>,
    pub hostname: Option<String>,
    pub unix: Option<String>,
    #[allow(deprecated)]
    pub routes: Option<JsObject>,
    pub fetch: Option<Function<'static, RequestInput, Promise<ResponsePayload>>>,
}

const ALL_METHODS: &[&str] = &["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];

fn collect_routes(value: Unknown) -> napi::Result<Vec<(Option<String>, RouteHandler)>> {
    match value.get_type()? {
        ValueType::Function => {
            let f: Function<'_, RequestInput, Promise<ResponsePayload>> = unsafe { value.cast() }?;
            let tsfn: RouteHandlerFn = f.build_threadsafe_function::<RequestInput>().build()?;
            Ok(vec![(None, RouteHandler::Dynamic(Arc::new(tsfn)))])
        }
        ValueType::Object => {
            #[allow(deprecated)]
            let obj: JsObject = unsafe { value.cast() }?;

            if let Ok(path) = obj.get_named_property::<String>("path") {
                if !obj.has_named_property("status")? {
                    return Ok(vec![(None, RouteHandler::File(path))]);
                }
            }

            if obj.has_named_property("status")? || obj.has_named_property("headers")? || obj.has_named_property("body")? {
                let status: Option<u32> = obj.get_named_property("status").ok();
                let headers: Option<HashMap<String, String>> = obj.get_named_property("headers").ok();
                let body: Option<Buffer> = obj.get_named_property("body").ok();
                return Ok(vec![(None, RouteHandler::Static(ResponsePayload {
                    status,
                    headers,
                    body,
                }))]);
            }

            let names = obj.get_property_names()?;
            let len = names.get_array_length()?;
            let mut out = Vec::new();
            for i in 0..len {
                let method: String = names.get_element(i)?;
                let v: Unknown = obj.get_named_property(&method)?;
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

static SHUTDOWN_HANDLE: OnceLock<Mutex<Option<RocketShutdownHandle>>> = OnceLock::new();

#[napi]
pub fn serve(options: ServeOptions) -> napi::Result<Server> {
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

    let mut rocket_routes: Vec<RocketRouteHandle> = Vec::new();
    if let Some(routes) = options.routes {
        let names = routes.get_property_names()?;
        let len = names.get_array_length()?;
        for i in 0..len {
            let pattern: String = names.get_element(i)?;
            let value: Unknown = routes.get_named_property(&pattern)?;
            for (method_opt, handler) in collect_routes(value)? {
                let methods: Vec<String> = method_opt
                    .map(|m| vec![m])
                    .unwrap_or_else(|| ALL_METHODS.iter().map(|s| s.to_string()).collect());
                for method in methods {
                    let m = Method::from_str(&method).map_err(|()| {
                        napi::Error::new(Status::InvalidArg, format!("invalid method: {method}"))
                    })?;
                    let translated = translate_pattern(&pattern);
                    rocket_routes.push(RocketRouteHandle::new(
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

    if let Some(fetch) = options.fetch {
        let tsfn: RouteHandlerFn = fetch.build_threadsafe_function::<RequestInput>().build()?;
        let tsfn = Arc::new(tsfn);
        for m in ALL_METHODS {
            rocket_routes.push(RocketRouteHandle::new(
                Method::from_str(m).unwrap(),
                "/<path..>",
                JsHandler {
                    handler: RouteHandler::Dynamic(tsfn.clone()),
                    pattern: "/<path..>".to_string(),
                },
            ));
        }
    }

    let mut config = rocket::config::Config::default();
    config.port = port;
    config.address = address;

    let rt = rocket::tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| napi::Error::new(Status::GenericFailure, e.to_string()))?;

    let rocket = rt
        .block_on(
            rocket::build()
                .configure(config)
                .mount("/", rocket_routes)
                .ignite(),
        )
        .map_err(|e| napi::Error::new(Status::GenericFailure, e.to_string()))?;

    let shutdown = rocket.shutdown();
    *SHUTDOWN_HANDLE.get_or_init(|| Mutex::new(None)).lock().unwrap() = Some(shutdown);

    std::thread::spawn(move || {
        let _ = rt.block_on(rocket.launch());
    });

    Ok(Server {
        url,
        port: port as u32,
    })
}

#[napi]
pub struct Server {
    pub url: String,
    pub port: u32,
}

#[napi]
impl Server {
    #[napi]
    pub fn stop(&self) -> napi::Result<()> {
        if let Some(shutdown) = SHUTDOWN_HANDLE.get().and_then(|m| m.lock().unwrap().take()) {
            shutdown.notify();
        }
        Ok(())
    }
}

struct RouteSpec {
    method: String,
    path: String,
    handler: RouteHandler,
}

static ROUTES: OnceLock<Mutex<Vec<RouteSpec>>> = OnceLock::new();

fn routes() -> &'static Mutex<Vec<RouteSpec>> {
    ROUTES.get_or_init(|| Mutex::new(Vec::new()))
}

#[napi]
pub fn register_route(
    method: String,
    path: String,
    handler: Function<'_, RequestInput, Promise<ResponsePayload>>,
) -> napi::Result<()> {
    let tsfn: RouteHandlerFn = handler.build_threadsafe_function::<RequestInput>().build()?;
    routes().lock().unwrap().push(RouteSpec {
        method,
        path,
        handler: RouteHandler::Dynamic(Arc::new(tsfn)),
    });
    Ok(())
}

#[napi]
pub async fn launch() -> napi::Result<()> {
    let specs = std::mem::take(&mut *routes().lock().unwrap());
    let mut rocket_routes: Vec<RocketRouteHandle> = Vec::with_capacity(specs.len());
    for spec in specs {
        let method = Method::from_str(&spec.method).map_err(|()| {
            napi::Error::new(Status::InvalidArg, format!("invalid method: {}", spec.method))
        })?;
        let translated = translate_pattern(&spec.path);
        rocket_routes.push(RocketRouteHandle::new(
            method,
            &translated,
            JsHandler {
                handler: spec.handler,
                pattern: translated.clone(),
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

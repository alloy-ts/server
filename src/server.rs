use std::collections::HashMap;
use std::io::Cursor;
use std::net::{IpAddr, Ipv4Addr, TcpListener};
use std::sync::{Arc, Mutex};

use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
use napi::Unknown;
use napi_derive::napi;

use rocket::config::Config;
use rocket::data::ByteUnit;
use rocket::fairing::AdHoc;
use rocket::http::{Header, Method, Status as RocketStatus};
use rocket::request::Request;
use rocket::response::Response as RocketResponse;
use rocket::route::{Handler, Outcome as RouteOutcome, Route};
use rocket::{Data, Shutdown};

type HandlerTsfn = ThreadsafeFunction<
    RequestInput,
    Either<ResponsePayload, Promise<ResponsePayload>>,
    (RequestInput,),
    Status,
    false,
    false,
    0,
>;

#[allow(deprecated)]
fn to_js_function(handler: Unknown<'_>) -> napi::JsFunction {
    unsafe { std::mem::transmute_copy::<Unknown, napi::JsFunction>(&handler) }
}

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
    body_data: Option<Vec<u8>>,
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
            status_code: init.status.unwrap_or(200),
            headers_list: init.headers.unwrap_or_default(),
            body_data: body,
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
            status_code: init.status.unwrap_or(200),
            headers_list: headers,
            body_data: Some(bytes),
        })
    }

    #[napi]
    pub fn redirect(url: String, status: Option<u32>) -> Self {
        Response {
            status_code: status.unwrap_or(302),
            headers_list: vec![("Location".to_string(), url)],
            body_data: None,
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
        self.body_data.as_ref().map(|b| Buffer::from(b.clone()))
    }
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
    File {
        path: String,
        cache: Arc<Mutex<Option<Vec<u8>>>>,
    },
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
                let output = match tsfn.call_async(input).await {
                    Ok(Either::A(payload)) => payload,
                    Ok(Either::B(promise)) => match promise.await {
                        Ok(payload) => payload,
                        Err(e) => {
                            eprintln!("JS promise error: {e:?}");
                            return RouteOutcome::Error(RocketStatus::InternalServerError);
                        }
                    },
                    Err(e) => {
                        eprintln!("TSFN call error: {e:?}");
                        return RouteOutcome::Error(RocketStatus::InternalServerError);
                    }
                };
                RouteOutcome::Success(build_response(&output))
            }
            RouteKind::Static(payload) => RouteOutcome::Success(build_response(payload.as_ref())),
            RouteKind::File { path, cache: _ } => match rocket::tokio::fs::read(path).await {
                Ok(bytes) => {
                    let mut builder = RocketResponse::build();
                    builder.sized_body(bytes.len(), Cursor::new(bytes));
                    RouteOutcome::Success(builder.finalize())
                }
                Err(_) => RouteOutcome::Error(RocketStatus::NotFound),
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

fn translate_pattern(pattern: &str) -> String {
    let mut out = String::with_capacity(pattern.len() + 1);
    if !pattern.starts_with('/') {
        out.push('/');
    }
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
    if let Some(status) = RocketStatus::from_code(payload.status as u16) {
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

fn mount_value<'env>(
    value: Unknown<'env>,
    pattern: &str,
    methods: &[Method],
    out: &mut Vec<Route>,
) -> napi::Result<()> {
    match value.get_type()? {
        ValueType::Function => {
            let tsfn: HandlerTsfn = to_js_function(value).create_threadsafe_function::<
                RequestInput,
                (RequestInput,),
                Either<ResponsePayload, Promise<ResponsePayload>>,
                Status,
                _,
                false,
                false,
                0,
            >(|ctx| Ok((ctx.value,)))?;
            push_kind(RouteKind::Fn(Arc::new(tsfn)), pattern, methods, out);
        }
        ValueType::Object => {
            let v = value;
            if let Ok(fr) = unsafe { v.cast::<StaticFile>() } {
                push_kind(
                    RouteKind::File {
                        path: fr.path,
                        cache: Arc::new(Mutex::new(None)),
                    },
                    pattern,
                    methods,
                    out,
                );
            } else if let Ok(resp) = unsafe { v.cast::<ResponsePayload>() } {
                push_kind(RouteKind::Static(Arc::new(resp)), pattern, methods, out);
            } else {
                let obj: Object = unsafe { v.cast()? };
                let names = obj.get_property_names()?;
                let len = names.get_array_length()?;
                for i in 0..len {
                    let method: String = names.get_element(i)?;
                    let method_value: Unknown<'_> = obj.get_named_property(&method)?;
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

fn push_kind(kind: RouteKind, pattern: &str, methods: &[Method], out: &mut Vec<Route>) {
    let handler = JsHandler(Arc::new(kind));
    let rocket_pattern = translate_pattern(pattern);
    for m in methods {
        out.push(Route::new(*m, &rocket_pattern, handler.clone()));
    }
}

type ShutdownSlot = Arc<Mutex<Option<Shutdown>>>;

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
    let mut port = options.port.unwrap_or(3000) as u16;
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

    if port == 0 {
        if let Ok(listener) = TcpListener::bind((address, 0)) {
            if let Ok(addr) = listener.local_addr() {
                port = addr.port();
            }
        }
    }

    let url = format!("http://{}:{}", hostname, port);

    let mut out: Vec<Route> = Vec::new();
    if let Some(table) = routes {
        let obj: Object = unsafe { table.cast()? };
        let names = obj.get_property_names()?;
        let len = names.get_array_length()?;
        for i in 0..len {
            let pattern: String = names.get_element(i)?;
            let value: Unknown<'_> = obj.get_named_property(&pattern)?;
            mount_value(value, &pattern, ALL_METHODS, &mut out)?;
        }
    }

    if let Some(fetch) = fetch {
        let tsfn: HandlerTsfn = to_js_function(fetch).create_threadsafe_function::<
            RequestInput,
            (RequestInput,),
            Either<ResponsePayload, Promise<ResponsePayload>>,
            Status,
            _,
            false,
            false,
            0,
        >(|ctx| Ok((ctx.value,)))?;
        let tsfn = Arc::new(tsfn);
        for m in ALL_METHODS {
            push_kind(RouteKind::Fn(tsfn.clone()), "/<_..>", &[*m], &mut out);
        }
    }

    let mut config = Config::default();
    config.port = port;
    config.address = address;

    let slot: ShutdownSlot = Arc::new(Mutex::new(None));
    let slot_clone = slot.clone();

    let rocket = rocket::build()
        .configure(config)
        .mount("/", out)
        .attach(AdHoc::on_liftoff("capture-shutdown", move |orbit| {
            Box::pin(async move {
                *slot_clone.lock().unwrap() = Some(orbit.shutdown());
            })
        }));

    let rt = rocket::tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| napi::Error::new(Status::GenericFailure, e.to_string()))?;

    std::thread::spawn(move || {
        rt.block_on(async move {
            let _ = rocket.launch().await;
        });
    });

    Ok(Server {
        url,
        port: port as u32,
        shutdown_slot: slot,
    })
}

#[napi(object)]
pub struct ServeOptions {
    pub port: Option<u32>,
    pub hostname: Option<String>,
    pub unix: Option<String>,
}

#[napi]
pub struct Server {
    pub url: String,
    pub port: u32,
    shutdown_slot: ShutdownSlot,
}

#[napi]
impl Server {
    #[napi]
    pub fn stop(&self) -> napi::Result<()> {
        if let Some(shutdown) = self.shutdown_slot.lock().unwrap().take() {
            shutdown.notify();
        }
        Ok(())
    }
}

static ROUTES: std::sync::OnceLock<Mutex<Vec<Route>>> = std::sync::OnceLock::new();

fn routes() -> &'static Mutex<Vec<Route>> {
    ROUTES.get_or_init(|| Mutex::new(Vec::new()))
}

#[napi]
pub fn register_route(method: String, path: String, handler: Unknown<'_>) -> napi::Result<()> {
    let tsfn: HandlerTsfn = to_js_function(handler).create_threadsafe_function::<
        RequestInput,
        (RequestInput,),
        Either<ResponsePayload, Promise<ResponsePayload>>,
        Status,
        _,
        false,
        false,
        0,
    >(|ctx| Ok((ctx.value,)))?;
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

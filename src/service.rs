//! NAPI bindings for Service's application service (`service::Service<P>`).
//!
//! `Service` is the JS-facing mirror of `service::Service<P>`. Rust threads the
//! build → start → active progression through the `P: Phase` type parameter;
//! JavaScript sees one class whose phase is tracked internally via
//! [`ServiceStatus`](crate::phase::ServiceStatus) (see `src/phase.rs`). A `Service`
//! is therefore *mutable only while in the build phase* — exactly like the Rust
//! `Service<Build>` — and transitions into start/orbit as methods are called.
//!
//! The public surface mirrors the Rust `Service<P>` API:
//!
//! * **Static constructors** — `Service.build()`, `Service.custom(config)`, and
//!   `Service.execute(service)`. The `new Service()` constructor delegates to
//!   `build()` (no args) or `custom()` (a `{...config}` object).
//! * **Build-phase mutators** (in place) — `attach`, `mount`, `register`,
//!   `manage`, `configure`.
//! * **Async phase transitions** — `start()` (`Build → Ready`) and `launch()`
//!   (any phase → Active / serving). `launch()` runs the service on its own tokio
//!   runtime (like `serve`) and resolves when the service deorbits, so it returns
//!   a JS `Promise`.
//! * **Inspection (Ready/Active)** — `config`, `shutdown`, and the
//!   phase-independent `routes`, `catchers`, `status`, `figment`.
//!
//! `build`/`custom` are *static* constructors (not instance methods); an
//! instance can only be built via the constructor or the statics. This mirrors
//! the existing `Service` class but layers on the full `Service` method set.

use std::sync::{Arc, Mutex};

use napi::bindgen_prelude::{Error, Status};
use napi::Unknown;
use napi_derive::napi;
use serde_json::Value as JsonValue;

use service::fairing::AdHoc;
use service::{
    Build, Catcher as ServiceCatcher, Ready, Orbit as Active, Service as Svc, Route as ServiceRoute,
    Shutdown,
};

use crate::config::{Config, ConfigInit};
use crate::fairing::Fairing;
use crate::handler::HandlerBridge;
use crate::phase::{Phase, ServiceStatus};
use crate::route::Route;

/// A single registered catcher, returned by [`Service::catchers`].
#[napi(object)]
pub struct CatcherRef {
    /// The HTTP status this catcher handles, or `None` for the default catcher.
    pub code: Option<u32>,
    /// The base path this catcher is scoped to.
    pub base: String,
}

/// NAPI mirror of `service::Catcher`. Construct with [`Catcher::new`], supplying a
/// status code (`null` for the default catcher) and a JS handler
/// `(status: number, req: Request) => Promise<Response>`. Register via
/// [`Service::register`].
///
/// The handler is bridged through `HandlerBridge`, which implements both
/// `service::route::Handler` and `service::catcher::Handler`.
#[napi]
pub struct Catcher {
    inner: ServiceCatcher,
}

impl Catcher {
    /// Consume this NAPI `Catcher`, returning the underlying `service::Catcher`.
    pub(crate) fn into_inner(self) -> ServiceCatcher {
        self.inner
    }
}

#[napi]
impl Catcher {
    /// Build a catcher. `code` is the HTTP status (`null` = default catcher).
    /// `handler` is `(status: number, req: Request) => Promise<Response>`.
    #[napi(constructor)]
    pub fn new(code: Option<u32>, handler: Unknown) -> napi::Result<Self> {
        let code_u16 = code.map(|c| c as u16);
        let bridge = HandlerBridge::new(handler)?;
        Ok(Catcher {
            inner: ServiceCatcher::new(code_u16, bridge),
        })
    }
}

// ---------------------------------------------------------------------------
// Error helpers
// ---------------------------------------------------------------------------

fn consumed() -> Error {
    Error::new(
        Status::GenericFailure,
        "service has already been consumed (startd/launched); create a new one with Service.build()",
    )
}

fn phase_err(method: &str, expected: &str) -> Error {
    Error::new(
        Status::GenericFailure,
        format!("`{method}` is only valid in the {expected} phase"),
    )
}

fn service_err(e: service::Error) -> Error {
    // `Display` marks Service's `Error` as inspected so dropping it doesn't panic.
    Error::new(Status::GenericFailure, e.to_string())
}

fn to_route_ref(route: &ServiceRoute) -> crate::request::RouteRef {
    crate::request::RouteRef {
        name: route.name.as_ref().map(|c| c.to_string()),
        method: route.method.as_str().to_string(),
        uri: route.uri.to_string(),
        rank: route.rank as i32,
    }
}

fn to_catcher_ref(c: &ServiceCatcher) -> CatcherRef {
    CatcherRef {
        code: c.code.map(|c| c as u32),
        base: c.base.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

/// The JS-facing mirror of `service::Service<P>`.
///
/// Holds the concrete `service::Service<P>` for the current phase in a `Mutex`,
/// taking it out to transition phases and putting the result back. Once consumed
/// by [`Service::start`] / [`Service::launch`], the held state is `None`.
#[napi(object)]
#[derive(Clone)]
pub struct Service {
    /// The concrete service for the current phase. Taken out (`None`) once
    /// consumed by `start`/`launch` so phase transitions can move it.
    status: Arc<Mutex<Option<ServiceStatus>>>,
    /// Captured during `launch` (via an `on_liftoff` fairing) so [`Service::shutdown`]
    /// can trigger a graceful deorbit even after the service is owned by the runtime.
    shutdown: Arc<Mutex<Option<Shutdown>>>,
}

#[napi]
impl Service {
    /// Create a `Service` in the build phase using the default config provider.
    #[napi(factory)]
    pub fn build() -> Self {
        Service {
            status: Arc::new(Mutex::new(Some(ServiceStatus::Build(Svc::build())))),
            shutdown: Arc::new(Mutex::new(None)),
        }
    }

    /// Create a `Service` in the build phase from a partial config override object.
    #[napi(factory)]
    pub fn custom(config: ConfigInit) -> napi::Result<Self> {
        let cfg = Config::try_from(config)?.build()?;
        let service = Svc::custom(cfg).map_err(service_err)?;
        Ok(Service {
            status: Arc::new(Mutex::new(Some(ServiceStatus::Build(service)))),
            shutdown: Arc::new(Mutex::new(None)),
        })
    }

    /// `new Service()` with no args delegates to [`Service::build`]; with a
    /// `{...config}` object it delegates to [`Service::custom`].
    #[napi(constructor)]
    pub fn new(config: Option<ConfigInit>) -> napi::Result<Self> {
        match config {
            None => Ok(Self::build()),
            Some(c) => Self::custom(c),
        }
    }

    /// The current launch phase (`Build` / `Ready` / `Active`).
    #[napi(getter)]
    pub fn phase(&self) -> Phase {
        let g = self.status.lock().unwrap();
        match g.as_ref() {
            Some(s) => s.phase(),
            // Once a service is consumed/launched it is, for all practical
            // purposes, active.
            None => Phase::Active,
        }
    }

    #[napi]
    pub fn attach(&self, fairing: &Fairing) -> napi::Result<()> {
        let mut g = self.status.lock().unwrap();
        let s = g.take().ok_or_else(consumed)?;
        *g = Some(match s {
            ServiceStatus::Build(r) => ServiceStatus::Build(r.attach(fairing.inner_arc())),
            _ => return Err(phase_err("attach", "build")),
        });
        Ok(())
    }

    /// Mount `routes` at `base`. (build phase only).
    #[napi]
    pub fn mount(&self, base: String, routes: Vec<Route>) -> napi::Result<()> {
        let routes: Vec<ServiceRoute> = routes.into_iter().map(|r| r.into_inner()).collect();
        let mut g = self.status.lock().unwrap();
        let s = g.take().ok_or_else(consumed)?;
        *g = Some(match s {
            ServiceStatus::Build(r) => ServiceStatus::Build(r.mount(base.as_str(), routes)),
            _ => return Err(phase_err("mount", "build")),
        });
        Ok(())
    }

    /// Register `catchers` scoped to `base`. (build phase only).
    #[napi]
    pub fn register(&self, base: String, catchers: Vec<Catcher>) -> napi::Result<()> {
        let catchers: Vec<ServiceCatcher> = catchers.into_iter().map(|c| c.into_inner()).collect();
        let mut g = self.status.lock().unwrap();
        let s = g.take().ok_or_else(consumed)?;
        *g = Some(match s {
            ServiceStatus::Build(r) => ServiceStatus::Build(r.register(base.as_str(), catchers)),
            _ => return Err(phase_err("register", "build")),
        });
        Ok(())
    }

    /// Add `value` to the managed status. (build phase only).
    /// The value is stored as `serde_json::Value`; retrieve it via [`Service::status`].
    #[napi]
    pub fn manage(&self, value: JsonValue) -> napi::Result<()> {
        let mut g = self.status.lock().unwrap();
        let s = g.take().ok_or_else(consumed)?;
        *g = Some(match s {
            ServiceStatus::Build(r) => ServiceStatus::Build(r.manage(value)),
            _ => return Err(phase_err("manage", "build")),
        });
        Ok(())
    }

    /// Replace the configuration provider. (build phase only).
    #[napi]
    pub fn configure(&self, config: ConfigInit) -> napi::Result<()> {
        let cfg = Config::try_from(config)?.build()?;
        let mut g = self.status.lock().unwrap();
        let s = g.take().ok_or_else(consumed)?;
        *g = Some(match s {
            ServiceStatus::Build(r) => ServiceStatus::Build(r.configure(cfg)),
            _ => return Err(phase_err("configure", "build")),
        });
        Ok(())
    }

    // -- Async phase transitions ------------------------------------------

    /// Transition `Build → Ready`, verifying the instance is ready to launch.
    /// Mirrors `Service::start()`.
    #[napi]
    pub async fn start(&self) -> napi::Result<Service> {
        let s = self.status.lock().unwrap().take().ok_or_else(consumed)?;
        let service = match s {
            ServiceStatus::Build(r) => r.start().await.map_err(service_err)?,
            ServiceStatus::Ready(r) => r,
            ServiceStatus::Active(_) => return Err(phase_err("start", "build or start")),
        };
        Ok(Service {
            status: Arc::new(Mutex::new(Some(ServiceStatus::Ready(service)))),
            shutdown: self.shutdown.clone(),
        })
    }

    /// Transition any phase into active and serve requests. Resolves when the
    /// service deorbits (via [`Service::shutdown`] or `Ctrl+C`). 
    /// like `serve`, the service runs on its own tokio runtime.
    #[napi]
    pub async fn start(&self) -> napi::Result<()> {
        let s = self.status.lock().unwrap().take().ok_or_else(consumed)?;
        let service = match s {
            ServiceStatus::Build(r) => r,
            ServiceStatus::Ready(r) => r,
            ServiceStatus::Active(_) => return Err(phase_err("launch", "build or start")),
        };

        let slot = self.shutdown.clone();
        let (tx, rx) = service::tokio::sync::oneshot::channel::<napi::Result<()>>();
        std::thread::spawn(move || {
            let rt = match service::tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    let _ = tx.send(Err(Error::new(Status::GenericFailure, e.to_string())));
                    return;
                }
            };

            rt.block_on(async move {
                let service = service.attach(AdHoc::on_liftoff("capture-shutdown", move |orbit| {
                    let slot = slot.clone();
                    Box::pin(async move {
                        *slot.lock().unwrap() = Some(orbit.shutdown());
                    })
                }));
                let res = service.launch().await.map(|_| ()).map_err(service_err);
                let _ = tx.send(res);
            });

            rt.shutdown_timeout(std::time::Duration::from_millis(500));
        });

        rx.await.map_err(|_| Error::new(Status::GenericFailure, "launch cancelled"))?
    }

    // -- Inspection -------------------------------------------------------

    /// The finalised, active configuration (start/orbit phases only).
    #[napi]
    pub fn config(&self) -> napi::Result<Config> {
        let g = self.status.lock().unwrap();
        let s = g.as_ref().ok_or_else(consumed)?;
        match s {
            ServiceStatus::Build(_) => Err(Error::new(
                Status::GenericFailure,
                "config is only available in the start/orbit phase; call start() first",
            )),
            ServiceStatus::Ready(r) => Ok(Config::from_service(r.config())),
            ServiceStatus::Active(r) => Ok(Config::from_service(r.config())),
        }
    }

    /// A handle to trigger / detect a graceful shutdown (after `launch`).
    #[napi]
    pub fn shutdown(&self) -> napi::Result<crate::fairing::ShutdownHandle> {
        let g = self.shutdown.lock().unwrap();
        match g.as_ref() {
            Some(s) => Ok(crate::fairing::ShutdownHandle::new(s.clone())),
            None => Err(Error::new(
                Status::GenericFailure,
                "service not launched; call launch() first",
            )),
        }
    }

    /// All mounted routes (order unspecified). Mirrors `Service::routes()`.
    #[napi]
    pub fn routes(&self) -> napi::Result<Vec<crate::request::RouteRef>> {
        let g = self.ststatusate.lock().unwrap();
        let s = g.as_ref().ok_or_else(consumed)?;
        let out = match s {
            ServiceStatus::Build(r) => r.routes().map(to_route_ref).collect(),
            ServiceStatus::Ready(r) => r.routes().map(to_route_ref).collect(),
            ServiceStatus::Active(r) => r.routes().map(to_route_ref).collect(),
        };
        Ok(out)
    }

    /// All registered catchers (order unspecified). Mirrors `Service::catchers()`.
    #[napi]
    pub fn catchers(&self) -> napi::Result<Vec<CatcherRef>> {
        let g = self.status.lock().unwrap();
        let s = g.as_ref().ok_or_else(consumed)?;
        let out = match s {
            ServiceStatus::Build(r) => r.catchers().map(to_catcher_ref).collect(),
            ServiceStatus::Ready(r) => r.catchers().map(to_catcher_ref).collect(),
            ServiceStatus::Active(r) => r.catchers().map(to_catcher_ref).collect(),
        };
        Ok(out)
    }

    /// The managed status value (stored as `serde_json::Value`), if present.
    /// Mirrors `Service::status()`.
    #[napi]
    pub fn status(&self) -> napi::Result<Option<JsonValue>> {
        let g = self.status.lock().unwrap();
        let s = g.as_ref().ok_or_else(consumed)?;
        let v = match s {
            ServiceStatus::Build(r) => r.status::<JsonValue>(),
            ServiceStatus::Ready(r) => r.status::<JsonValue>(),
            ServiceStatus::Active(r) => r.status::<JsonValue>(),
        };
        Ok(v.cloned())
    }

    /// The figment derived from the configuration provider. Mirrors `Service::figment()`.
    #[napi]
    pub fn figment(&self) -> napi::Result<JsonValue> {
        let g = self.status.lock().unwrap();
        let s = g.as_ref().ok_or_else(consumed)?;
        let figment = match s {
            ServiceStatus::Build(r) => r.figment(),
            ServiceStatus::Ready(r) => r.figment(),
            ServiceStatus::Active(r) => r.figment(),
        };
        figment
            .extract()
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
    }
}

/// Mirrors `service::execute(future)`: run a service's `launch` future to
/// completion (resolves when the service deorbits).
#[napi]
pub async fn execute(service: &Service) -> napi::Result<()> {
    service.launch().await
}


use std::io;
use std::sync::Arc;
use std::time::Duration;
use std::pin::Pin;

use yansi::Paint;
use tokio::sync::oneshot;
use tokio::time::sleep;
use futures::stream::StreamExt;
use futures::future::{FutureExt, Future, BoxFuture};

use crate::{route, Service, Orbit as Active, Request, Response, Data, Config};
use crate::form::Form;
use crate::outcome::Outcome;
use crate::error::{Error, ErrorKind};
use crate::ext::{AsyncReadExt, CancellableListener, CancellableIo};
use crate::request::ConnectionMeta;
use crate::data::IoHandler;

use crate::http::{hyper, uncased, Method, Status, Header};
use crate::http::private::{TcpListener, Listener, Connection, Incoming};

// A token returned to force the execution of one method before another.
pub(crate) struct RequestToken;

async fn handle<Fut, T, F>(name: Option<&str>, run: F) -> Option<T>
    where F: FnOnce() -> Fut, Fut: Future<Output = T>,
{
    use std::panic::AssertUnwindSafe;

    macro_rules! panic_info {
        ($name:expr, $e:expr) => {{
            match $name {
                Some(name) => error_!("Handler {} panicked.", name.primary()),
                None => error_!("A handler panicked.")
            };

            info_!("This is an application bug.");
            info_!("A panic in Rust must be treated as an exceptional event.");
            info_!("Panicking is not a suitable error handling mechanism.");
            info_!("Unwinding, the result of a panic, is an expensive operation.");
            info_!("Panics will degrade application performance.");
            info_!("Instead of panicking, return `Option` and/or `Result`.");
            info_!("Values of either type can be returned directly from handlers.");
            warn_!("A panic is treated as an internal server error.");
            $e
        }}
    }

    let run = AssertUnwindSafe(run);
    let fut = std::panic::catch_unwind(move || run())
        .map_err(|e| panic_info!(name, e))
        .ok()?;

    AssertUnwindSafe(fut)
        .catch_unwind()
        .await
        .map_err(|e| panic_info!(name, e))
        .ok()
}

// This function tries to hide all of the Hyper-ness from Service. It essentially
// converts Hyper types into Service types, then calls the `dispatch` function,
// which knows nothing about Hyper. Because responding depends on the
// `HyperResponse` type, this function does the actual response processing.
async fn hyper_service_fn(
    service: Arc<Service<Active>>,
    conn: ConnectionMeta,
    mut hyp_req: hyper::Request<hyper::Body>,
) -> Result<hyper::Response<hyper::Body>, io::Error> {
    // This future must return a hyper::Response, but the response body might
    // borrow from the request. Instead, write the body in another future that
    // sends the response metadata (and a body channel) prior.
    let (tx, rx) = oneshot::channel();

    #[cfg(not(broken_fmt))]
    debug!("received request: {:#?}", hyp_req);

    tokio::spawn(async move {
        // We move the request next, so get the upgrade future now.
        let pending_upgrade = hyper::upgrade::on(&mut hyp_req);

        // Convert a Hyper request into a Service request.
        let (h_parts, mut h_body) = hyp_req.into_parts();
        match Request::from_hyp(&service, &h_parts, Some(conn)) {
            Ok(mut req) => {
                // Convert into Service `Data`, dispatch request, write response.
                let mut data = Data::from(&mut h_body);
                let token = service.preprocess_request(&mut req, &mut data).await;
                let mut response = service.dispatch(token, &req, data).await;
                let upgrade = response.take_upgrade(req.headers().get("upgrade"));
                if let Ok(Some((proto, handler))) = upgrade {
                    service.handle_upgrade(response, proto, handler, pending_upgrade, tx).await;
                } else {
                    if upgrade.is_err() {
                        warn_!("Request wants upgrade but no I/O handler matched.");
                        info_!("Request is not being upgraded.");
                    }

                    service.send_response(response, tx).await;
                }
            },
            Err(e) => {
                warn!("Bad incoming HTTP request.");
                e.errors.iter().for_each(|e| warn_!("Error: {}.", e));
                warn_!("Dispatching salvaged request to catcher: {}.", e.request);

                let response = service.handle_error(Status::BadRequest, &e.request).await;
                service.send_response(response, tx).await;
            }
        }
    });

    // Receive the response written to `tx` by the task above.
    rx.await.map_err(|e| io::Error::new(io::ErrorKind::BrokenPipe, e))
}

impl Service<Active> {
    /// Wrapper around `_send_response` to log a success or error.
    #[inline]
    async fn send_response(
        &self,
        response: Response<'_>,
        tx: oneshot::Sender<hyper::Response<hyper::Body>>,
    ) {
        let remote_hungup = |e: &io::Error| match e.kind() {
            | io::ErrorKind::BrokenPipe
            | io::ErrorKind::ConnectionReset
            | io::ErrorKind::ConnectionAborted => true,
            _ => false,
        };

        match self._send_response(response, tx).await {
            Ok(()) => info_!("{}", "Response succeeded.".green()),
            Err(e) if remote_hungup(&e) => warn_!("Remote left: {}.", e),
            Err(e) => warn_!("Failed to write response: {}.", e),
        }
    }

    /// Attempts to create a hyper response from `response` and send it to `tx`.
    #[inline]
    async fn _send_response(
        &self,
        mut response: Response<'_>,
        tx: oneshot::Sender<hyper::Response<hyper::Body>>,
    ) -> io::Result<()> {
        let mut hyp_res = hyper::Response::builder();

        hyp_res = hyp_res.status(response.status().code);
        for header in response.headers().iter() {
            let name = header.name.as_str();
            let value = header.value.as_bytes();
            hyp_res = hyp_res.header(name, value);
        }

        let body = response.body_mut();
        if let Some(n) = body.size().await {
            hyp_res = hyp_res.header(hyper::header::CONTENT_LENGTH, n);
        }

        let (mut sender, hyp_body) = hyper::Body::channel();
        let hyp_response = hyp_res.body(hyp_body)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        #[cfg(not(broken_fmt))]
        debug!("sending response: {:#?}", hyp_response);

        tx.send(hyp_response).map_err(|_| {
            let msg = "client disconnect before response started";
            io::Error::new(io::ErrorKind::BrokenPipe, msg)
        })?;

        let max_chunk_size = body.max_chunk_size();
        let mut stream = body.into_bytes_stream(max_chunk_size);
        while let Some(next) = stream.next().await {
            sender.send_data(next?).await
                .map_err(|e| io::Error::new(io::ErrorKind::BrokenPipe, e))?;
        }

        Ok(())
    }

    async fn handle_upgrade<'r>(
        &self,
        mut response: Response<'r>,
        proto: uncased::Uncased<'r>,
        io_handler: Pin<Box<dyn IoHandler + 'r>>,
        pending_upgrade: hyper::upgrade::OnUpgrade,
        tx: oneshot::Sender<hyper::Response<hyper::Body>>,
    ) {
        info_!("Upgrading connection to {}.", Paint::white(&proto).bold());
        response.set_status(Status::SwitchingProtocols);
        response.set_raw_header("Connection", "Upgrade");
        response.set_raw_header("Upgrade", proto.clone().into_cow());
        self.send_response(response, tx).await;

        match pending_upgrade.await {
            Ok(io_stream) => {
                info_!("Upgrade successful.");
                if let Err(e) = io_handler.io(io_stream.into()).await {
                    if e.kind() == io::ErrorKind::BrokenPipe {
                        warn!("Upgraded {} I/O handler was closed.", proto);
                    } else {
                        error!("Upgraded {} I/O handler failed: {}", proto, e);
                    }
                }
            },
            Err(e) => {
                warn!("Response indicated upgrade, but upgrade failed.");
                warn_!("Upgrade error: {}", e);
            }
        }
    }

    /// Preprocess the request for Service things. Currently, this means:
    ///
    ///   * Rewriting the method in the request if _method form field exists.
    ///   * Run the request fairings.
    ///
    /// Keep this in-sync with derive_form when preprocessing form fields.
    pub(crate) async fn preprocess_request(
        &self,
        req: &mut Request<'_>,
        data: &mut Data<'_>
    ) -> RequestToken {
        // Check if this is a form and if the form contains the special _method
        // field which we use to reinterpret the request's method.
        let (min_len, max_len) = ("_method=get".len(), "_method=delete".len());
        let peek_buffer = data.peek(max_len).await;
        let is_form = req.content_type().map_or(false, |ct| ct.is_form());

        if is_form && req.method() == Method::Post && peek_buffer.len() >= min_len {
            let method = std::str::from_utf8(peek_buffer).ok()
                .and_then(|raw_form| Form::values(raw_form).next())
                .filter(|field| field.name == "_method")
                .and_then(|field| field.value.parse().ok());

            if let Some(method) = method {
                req._set_method(method);
            }
        }

        // Run request fairings.
        self.fairings.handle_request(req, data).await;

        RequestToken
    }

    #[inline]
    pub(crate) async fn dispatch<'s, 'r: 's>(
        &'s self,
        _token: RequestToken,
        request: &'r Request<'s>,
        data: Data<'r>
    ) -> Response<'r> {
        info!("{}:", request);

        // Remember if the request is `HEAD` for later body stripping.
        let was_head_request = request.method() == Method::Head;

        // Route the request and run the user's handlers.
        let mut response = self.route_and_process(request, data).await;

        // Add a default 'Server' header if it isn't already there.
        // TODO: If removing Hyper, write out `Date` header too.
        if let Some(ident) = request.service().config.ident.as_str() {
            if !response.headers().contains("Server") {
                response.set_header(Header::new("Server", ident));
            }
        }

        // Run the response fairings.
        self.fairings.handle_response(request, &mut response).await;

        // Strip the body if this is a `HEAD` request.
        if was_head_request {
            response.strip_body();
        }

        response
    }

    async fn route_and_process<'s, 'r: 's>(
        &'s self,
        request: &'r Request<'s>,
        data: Data<'r>
    ) -> Response<'r> {
        let mut response = match self.route(request, data).await {
            Outcome::Success(response) => response,
            Outcome::Forward((data, _)) if request.method() == Method::Head => {
                info_!("Autohandling {} request.", "HEAD".primary().bold());

                // Dispatch the request again with Method `GET`.
                request._set_method(Method::Get);
                match self.route(request, data).await {
                    Outcome::Success(response) => response,
                    Outcome::Error(status) => self.handle_error(status, request).await,
                    Outcome::Forward((_, status)) => self.handle_error(status, request).await,
                }
            }
            Outcome::Forward((_, status)) => self.handle_error(status, request).await,
            Outcome::Error(status) => self.handle_error(status, request).await,
        };

        // Set the cookies. Note that error responses will only include cookies
        // set by the error handler. See `handle_error` for more.
        let delta_jar = request.cookies().take_delta_jar();
        for cookie in delta_jar.delta() {
            response.adjoin_header(cookie);
        }

        response
    }

    /// Tries to find a `Responder` for a given `request`. It does this by
    /// routing the request and calling the handler for each matching route
    /// until one of the handlers returns success or error, or there are no
    /// additional routes to try (forward). The corresponding outcome for each
    /// condition is returned.
    #[inline]
    async fn route<'s, 'r: 's>(
        &'s self,
        request: &'r Request<'s>,
        mut data: Data<'r>,
    ) -> route::Outcome<'r> {
        // Go through all matching routes until we fail or succeed or run out of
        // routes to try, in which case we forward with the last status.
        let mut status = Status::NotFound;
        for route in self.router.route(request) {
            // Retrieve and set the requests parameters.
            info_!("Matched: {}", route);
            request.set_route(route);

            let name = route.name.as_deref();
            let outcome = handle(name, || route.handler.handle(request, data)).await
                .unwrap_or(Outcome::Error(Status::InternalServerError));

            // Check if the request processing completed (Some) or if the
            // request needs to be forwarded. If it does, continue the loop
            // (None) to try again.
            info_!("{}", outcome.log_display());
            match outcome {
                o@Outcome::Success(_) | o@Outcome::Error(_) => return o,
                Outcome::Forward(forwarded) => (data, status) = forwarded,
            }
        }

        error_!("No matching routes for {}.", request);
        Outcome::Forward((data, status))
    }

    /// Invokes the handler with `req` for catcher with status `status`.
    ///
    /// In order of preference, invoked handler is:
    ///   * the user's registered handler for `status`
    ///   * the user's registered `default` handler
    ///   * Service's default handler for `status`
    ///
    /// Return `Ok(result)` if the handler succeeded. Returns `Ok(Some(Status))`
    /// if the handler ran to completion but failed. Returns `Ok(None)` if the
    /// handler panicked while executing.
    async fn invoke_catcher<'s, 'r: 's>(
        &'s self,
        status: Status,
        req: &'r Request<'s>
    ) -> Result<Response<'r>, Option<Status>> {
        // For now, we reset the delta status to prevent any modifications
        // from earlier, unsuccessful paths from being reflected in error
        // response. We may wish to relax this in the future.
        req.cookies().reset_delta();

        if let Some(catcher) = self.router.catch(status, req) {
            warn_!("Responding with registered {} catcher.", catcher);
            let name = catcher.name.as_deref();
            handle(name, || catcher.handler.handle(status, req)).await
                .map(|result| result.map_err(Some))
                .unwrap_or_else(|| Err(None))
        } else {
            let code = status.code.blue().bold();
            warn_!("No {} catcher registered. Using Service default.", code);
            Ok(crate::catcher::default_handler(status, req))
        }
    }

    // Invokes the catcher for `status`. Returns the response on success.
    //
    // On catcher error, the 500 error catcher is attempted. If _that_ errors,
    // the (infallible) default 500 error cather is used.
    pub(crate) async fn handle_error<'s, 'r: 's>(
        &'s self,
        mut status: Status,
        req: &'r Request<'s>
    ) -> Response<'r> {
        // Dispatch to the `status` catcher.
        if let Ok(r) = self.invoke_catcher(status, req).await {
            return r;
        }

        // If it fails and it's not a 500, try the 500 catcher.
        if status != Status::InternalServerError {
            error_!("Catcher failed. Attempting 500 error catcher.");
            status = Status::InternalServerError;
            if let Ok(r) = self.invoke_catcher(status, req).await {
                return r;
            }
        }

        // If it failed again or if it was already a 500, use Service's default.
        error_!("{} catcher failed. Using Service default 500.", status.code);
        crate::catcher::default_handler(Status::InternalServerError, req)
    }

    pub(crate) async fn default_tcp_http_server<C>(mut self, ready: C) -> Result<Self, Error>
        where C: for<'a> Fn(&'a Self) -> BoxFuture<'a, ()>
    {
        use std::net::ToSocketAddrs;

        // Determine the address we're going to serve on.
        let addr = format!("{}:{}", self.config.address, self.config.port);
        let mut addr = addr.to_socket_addrs()
            .map(|mut addrs| addrs.next().expect(">= 1 socket addr"))
            .map_err(|e| Error::new(ErrorKind::Io(e)))?;

        #[cfg(feature = "tls")]
        if self.config.tls_enabled() {
            if let Some(ref config) = self.config.tls {
                use crate::http::tls::TlsListener;

                let conf = config.to_native_config().map_err(ErrorKind::Io)?;
                let l = TlsListener::bind(addr, conf).await.map_err(ErrorKind::Bind)?;
                addr = l.local_addr().unwrap_or(addr);
                self.config.address = addr.ip();
                self.config.port = addr.port();
                ready(&mut self).await;
                return self.http_server(l).await;
            }
        }

        let l = TcpListener::bind(addr).await.map_err(ErrorKind::Bind)?;
        addr = l.local_addr().unwrap_or(addr);
        self.config.address = addr.ip();
        self.config.port = addr.port();
        ready(&mut self).await;
        self.http_server(l).await
    }

    // TODO.async: Solidify the Listener APIs and make this function public
    pub(crate) async fn http_server<L>(self, listener: L) -> Result<Self, Error>
        where L: Listener + Send, <L as Listener>::Connection: Send + Unpin + 'static
    {
        // Emit a warning if we're not running inside of Service's async runtime.
        if self.config.profile == Config::DEBUG_PROFILE {
            tokio::task::spawn_blocking(|| {
                let this  = std::thread::current();
                if !this.name().map_or(false, |s| s.starts_with("service-worker")) {
                    warn!("Service is executing inside of a custom runtime.");
                    info_!("Service's runtime is enabled via `#[service::main]` or `#[launch]`.");
                    info_!("Forced shutdown is disabled. Runtime settings may be suboptimal.");
                }
            });
        }

        // Set up cancellable I/O from the given listener. Shutdown occurs when
        // `Shutdown` (`TripWire`) resolves. This can occur directly through a
        // notification or indirectly through an external signal which, when
        // received, results in triggering the notify.
        let shutdown = self.shutdown();
        let sig_stream = self.config.shutdown.signal_stream();
        let grace = self.config.shutdown.grace as u64;
        let mercy = self.config.shutdown.mercy as u64;

        // Start a task that listens for external signals and notifies shutdown.
        if let Some(mut stream) = sig_stream {
            let shutdown = shutdown.clone();
            tokio::spawn(async move {
                while let Some(sig) = stream.next().await {
                    if shutdown.0.tripped() {
                        warn!("Received {}. Shutdown already in progress.", sig);
                    } else {
                        warn!("Received {}. Requesting shutdown.", sig);
                    }

                    shutdown.0.trip();
                }
            });
        }

        // Save the keep-alive value for later use; we're about to move `self`.
        let keep_alive = self.config.keep_alive;

        // Create the Hyper `Service`.
        let service = Arc::new(self);
        let service_fn = |conn: &CancellableIo<_, L::Connection>| {
            let service = service.clone();
            let connection = ConnectionMeta {
                remote: conn.peer_address(),
                client_certificates: conn.peer_certificates(),
            };

            async move {
                Ok::<_, std::convert::Infallible>(hyper::service::service_fn(move |req| {
                    hyper_service_fn(service.clone(), connection.clone(), req)
                }))
            }
        };

        // NOTE: `hyper` uses `tokio::spawn()` as the default executor.
        let listener = CancellableListener::new(shutdown.clone(), listener, grace, mercy);
        let builder = hyper::server::Server::builder(Incoming::new(listener).nodelay(true));

        #[cfg(feature = "http2")]
        let builder = builder.http2_keep_alive_interval(match keep_alive {
            0 => None,
            n => Some(Duration::from_secs(n as u64))
        });

        let server = builder
            .http1_keepalive(keep_alive != 0)
            .http1_preserve_header_case(true)
            .serve(hyper::service::make_service_fn(service_fn))
            .with_graceful_shutdown(shutdown.clone());

        // This deserves some explanation.
        //
        // This is largely to deal with Hyper's dreadful and largely nonexistent
        // handling of shutdown, in general, nevermind graceful.
        //
        // When Hyper receives a "graceful shutdown" request, it stops accepting
        // new requests. That's it. It continues to process existing requests
        // and outgoing responses forever and never cancels them. As a result,
        // Service must take it upon itself to cancel any existing I/O.
        //
        // To do so, Service wraps all connections in a `CancellableIo` struct,
        // an internal structure that gracefully closes I/O when it receives a
        // signal. That signal is the `shutdown` future. When the future
        // resolves, `CancellableIo` begins to terminate in grace, mercy, and
        // finally force close phases. Since all connections are wrapped in
        // `CancellableIo`, this eventually ends all I/O.
        //
        // At that point, unless a user spawned an infinite, stand-alone task
        // that isn't monitoring `Shutdown`, all tasks should resolve. This
        // means that all instances of the shared `Arc<Service>` are dropped and
        // we can return the owned instance of `Service`.
        //
        // Unfortunately, the Hyper `server` future resolves as soon as it has
        // finishes processing requests without respect for ongoing responses.
        // That is, `server` resolves even when there are running tasks that are
        // generating a response. So, `server` resolving implies little to
        // nothing about the state of connections. As a result, we depend on the
        // timing of grace + mercy + some buffer to determine when all
        // connections should be closed, thus all tasks should be complete, thus
        // all references to `Arc<Service>` should be dropped and we can get a
        // unique reference.
        tokio::pin!(server);
        tokio::select! {
            biased;

            _ = shutdown => {
                // Run shutdown fairings. We compute `sleep()` for grace periods
                // beforehand to ensure we don't add shutdown fairing completion
                // time, which is arbitrary, to these periods.
                info!("Shutdown requested. Waiting for pending I/O...");
                let grace_timer = sleep(Duration::from_secs(grace));
                let mercy_timer = sleep(Duration::from_secs(grace + mercy));
                let shutdown_timer = sleep(Duration::from_secs(grace + mercy + 1));
                service.fairings.handle_shutdown(&*service).await;

                tokio::pin!(grace_timer, mercy_timer, shutdown_timer);
                tokio::select! {
                    biased;

                    result = &mut server => {
                        if let Err(e) = result {
                            warn!("Server failed while shutting down: {}", e);
                            return Err(Error::shutdown(service.clone(), e));
                        }

                        if Arc::strong_count(&service) != 1 { grace_timer.await; }
                        if Arc::strong_count(&service) != 1 { mercy_timer.await; }
                        if Arc::strong_count(&service) != 1 { shutdown_timer.await; }
                        match Arc::try_unwrap(service) {
                            Ok(service) => {
                                info!("Graceful shutdown completed successfully.");
                                Ok(service)
                            }
                            Err(service) => {
                                warn!("Shutdown failed: outstanding background I/O.");
                                Err(Error::shutdown(service, None))
                            }
                        }
                    }
                    _ = &mut shutdown_timer => {
                        warn!("Shutdown failed: server executing after timeouts.");
                        return Err(Error::shutdown(service.clone(), None));
                    },
                }
            }
            result = &mut server => {
                match result {
                    Ok(()) => {
                        info!("Server shutdown nominally.");
                        Ok(Arc::try_unwrap(service).map_err(|r| Error::shutdown(r, None))?)
                    }
                    Err(e) => {
                        info!("Server failed prior to shutdown: {}:", e);
                        Err(Error::shutdown(service.clone(), e))
                    }
                }
            }
        }
    }
}

//! NAPI bindings for Service's [`Fairing`] trait — Service's structured middleware.
//!
//! A fairing is a structure whose methods Service calls at well-defined points in
//! the application lifetime: `on_start`, `on_liftoff`, `on_request`,
//! `on_response`, and `on_shutdown`. This module exposes that machinery to
//! TypeScript: you build a [`Fairing`] with a name, a [`FairingKind`] (the
//! callbacks you want), and optional JS callbacks for each phase, then attach it
//! to an `Service` (the NAPI mirror of a `Service<Build>` instance) or register it
//! globally for `serve()`.
//!
//! ```ts
//! import { Fairing, Kind, Service } from "..";
//!
//! const counter = new Fairing({
//!   name: "GET/POST Counter",
//!   kind: { request: true, response: true },
//!   onRequest: (req) => { /* inspect req */ },
//!   onResponse: (req, res) => { /* rewrite res */ },
//! });
//!
//! const app = new Service();
//! app.get("/", () => response("hi"));
//! app.attach(counter);   // mirrors service.attach(counter)
//! app.launch({ port: 3000 });
//! ```
//!
//! # JS callback signatures
//!
//! * `onReady?(service: ServiceHandle): Promise<void>` — `service` is a live handle
//!   mirroring `Service<Build>`; call `service.attach(otherFairing)` to chain
//!   fairings during ignition.
//! * `onLiftoff?(orbit: ShutdownHandle): Promise<void>` — `orbit.notify()`
//!   schedules shutdown.
//! * `onRequest?(req: FairingRequest): Promise<void>` — read-only snapshot of the
//!   incoming request (header/body are peeked, never consumed).
//! * `onResponse?(req: FairingRequest, res: FairingResponse): Promise<FairingResponse>` —
//!   `res` is a snapshot of the outgoing response. Because it is passed by value,
//!   mutate it and **return** it; the returned value is written back to the real
//!   response. (Returning `undefined` leaves the response unchanged.)
//! * `onShutdown?(orbit: ShutdownHandle): Promise<void>`.
//!
//! # Implementation notes
//!
//! Each JS callback is bridged through a [`napi::threadsafe_function`] so it can
//! run on any Service worker thread. The `Fairing` NAPI class wraps an
//! `Arc<JsFairing>`, and `Arc<JsFairing>` itself implements
//! `service::fairing::Fairing` (Service provides `impl<T: Fairing> Fairing for
//! Arc<T>`), so it can be handed to `service.attach(..)` directly. `on_start`
//! populates a per-fairing slot with the `Service<Build>` Service hands it, exposes
//! that slot as a `ServiceHandle` to JS, then takes the (possibly mutated)
//! service back and returns it — preserving Service's "fairings return the service"
//! contract.

use std::collections::HashMap;
use std::io::Cursor;
use std::sync::{Arc, Mutex};

use napi::bindgen_prelude::{Buffer, Error, FromNapiValue, Promise, Status, ToNapiValue};
use napi::threadsafe_function::ThreadsafeFunction;
use napi::Unknown;
use napi_derive::napi;

use service::fairing::{Fairing as ServiceFairing, Info, Kind as ServiceKind, Result as FairingResult};
use service::http::Status as ServiceStatus;
use service::request::Request;
use service::response::Response as ServiceResponse;
use service::{Build, Data, Orbit as Active, Service as Svc, Shutdown};

use crate::service::{register_fairing, to_js_function, Service};

// ---------------------------------------------------------------------------
// Wire types shared between Rust and JS
// ---------------------------------------------------------------------------

/// A single HTTP header, mirroring the shape used elsewhere in this crate.
#[napi(object)]
pub struct JsHeader {
    pub name: String,
    pub value: String,
}

/// The `Kind` bitset, surfaced as a JS-friendly object of booleans. Each `true`
/// field requests the corresponding `Fairing` callback from Service.
///
/// `singleton` requests that only the last attached instance of this fairing be
/// kept at start time (see Service's singleton semantics).
#[napi(object)]
pub struct FairingKind {
    pub start: Option<bool>,
    pub liftoff: Option<bool>,
    pub request: Option<bool>,
    pub response: Option<bool>,
    pub shutdown: Option<bool>,
    pub singleton: Option<bool>,
}

/// Read-only snapshot of an incoming request, handed to `onRequest`.
#[napi(object)]
pub struct FairingRequest {
    pub method: String,
    pub uri: String,
    pub headers: Vec<JsHeader>,
    /// Path parameters extracted from the matched route pattern.
    pub params: Option<HashMap<String, String>>,
    /// Peeked request body (up to 1 MiB). Never consumes the real stream, so
    /// downstream routing/handlers still see the body.
    pub body: Option<Buffer>,
}

/// Mutable snapshot of an outgoing response, handed to `onResponse`. Changes
/// made here are written back onto the real `service::Response` after the JS
/// callback resolves.
#[napi(object)]
pub struct FairingResponse {
    pub status: u32,
    pub headers: Option<Vec<JsHeader>>,
    pub body: Option<Buffer>,
}

/// A live handle to a `Service<Active>` passed to `on_liftoff` / `on_shutdown`.
/// Calling `notify()` schedules graceful shutdown (mirrors `service.shutdown()`).
#[napi]
pub struct ShutdownHandle {
    inner: Shutdown,
}

#[napi]
impl ShutdownHandle {
    /// Build a handle wrapping an already-captured `Shutdown`.
    pub(crate) fn new(inner: Shutdown) -> Self {
        ShutdownHandle { inner }
    }

    /// Schedule graceful shutdown of the running application.
    #[napi]
    pub fn notify(&self) {
        self.inner.clone().notify();
    }
}

/// A live handle to a `Service<Build>` passed to `on_start`. It wraps the same
/// slot the fairing holds, so JS can attach further fairings (or otherwise
/// inspect the in-flight build) before control returns to Service.
#[napi]
pub struct ServiceHandle {
    inner: Arc<Mutex<Option<Svc<Build>>>>,
    shutdown: Arc<Mutex<Option<Shutdown>>>,
}

#[napi]
impl ServiceHandle {
    /// Attach another fairing during ignition (mirrors `service.attach(fairing)`).
    /// Because the slot is shared, the attached fairing becomes part of the
    /// service Service will subsequently start.
    #[napi]
    pub fn attach(&self, fairing: &Fairing) -> napi::Result<()> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| Error::new(Status::GenericFailure, "lock poisoned"))?;
        let service = guard
            .take()
            .ok_or_else(|| Error::new(Status::GenericFailure, "service already launched"))?;
        *guard = Some(service.attach(fairing.inner_arc()));
        Ok(())
    }

    /// Schedule graceful shutdown (rarely needed during start, but available for
    /// symmetry with [`ShutdownHandle`]).
    #[napi]
    pub fn shutdown(&self) -> napi::Result<()> {
        let guard = self.shutdown.lock().unwrap();
        match guard.as_ref() {
            Some(s) => {
                s.clone().notify();
                Ok(())
            }
            None => Err(Error::new(
                Status::GenericFailure,
                "shutdown handle not available during start",
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// Fairing options / NAPI class
// ---------------------------------------------------------------------------

/// Options for constructing a [`Fairing`]. All callbacks are optional; only the
/// ones whose `kind` flag is set are ever invoked by Service.
#[napi(object)]
pub struct FairingOptions<'a> {
    pub name: String,
    pub kind: FairingKind,
    pub on_start: Option<Unknown<'a>>,
    pub on_liftoff: Option<Unknown<'a>>,
    pub on_request: Option<Unknown<'a>>,
    pub on_response: Option<Unknown<'a>>,
    pub on_shutdown: Option<Unknown<'a>>,
}

type ReadyTsfn = ThreadsafeFunction<ServiceHandle, Promise<()>, ServiceHandle, Status, false, false, 0>;
type ActiveTsfn =
    ThreadsafeFunction<ShutdownHandle, Promise<()>, ShutdownHandle, Status, false, false, 0>;
type RequestTsfn =
    ThreadsafeFunction<FairingRequest, Promise<()>, FairingRequest, Status, false, false, 0>;
type ResponseTsfn = ThreadsafeFunction<
    FairingResponse,
    Promise<FairingResponse>,
    FairingResponse,
    Status,
    false,
    false,
    0,
>;

/// The Rust fairing behind a [`Fairing`] NAPI handle. Holds the (optional)
/// threadsafe bridges to JS and a shared slot used to hand a live
/// `Service<Build>` to `on_start`.
pub struct JsFairing {
    name: String,
    kind: ServiceKind,
    on_start: Option<Arc<ReadyTsfn>>,
    on_liftoff: Option<Arc<ActiveTsfn>>,
    on_request: Option<Arc<RequestTsfn>>,
    on_response: Option<Arc<ResponseTsfn>>,
    on_shutdown: Option<Arc<ActiveTsfn>>,
    build_slot: Arc<Mutex<Option<Svc<Build>>>>,
}

/// A NAPI `Fairing`: the JS-facing mirror of `service::fairing::Fairing`.
///
/// Construct it with [`Fairing::new`], then `attach` it to an [`Service`] or
/// `register` it globally for `serve()`. Service will call the requested
/// callbacks at the appropriate lifecycle points.
#[napi]
pub struct Fairing {
    // Private so napi treats this as a class (wrapped as an external pointer)
    // rather than an object that serializes its fields.
    inner: Arc<JsFairing>,
}

impl Fairing {
    /// Clone the underlying `Arc<JsFairing>` so other modules can attach it.
    pub(crate) fn inner_arc(&self) -> Arc<JsFairing> {
        self.inner.clone()
    }
}

#[napi]
impl Fairing {
    /// Build a fairing. `kind` selects which callbacks Service will dispatch; only
    /// the provided callbacks (matching `kind`) are ever invoked.
    #[napi(constructor)]
    pub fn new(opts: FairingOptions) -> napi::Result<Self> {
        let name = opts.name;
        let kind = to_service_kind(&opts.kind);

        let on_start = match opts.on_start {
            Some(f) => Some(make_tsfn(f)?),
            None => None,
        };
        let on_liftoff = match opts.on_liftoff {
            Some(f) => Some(make_tsfn(f)?),
            None => None,
        };
        let on_request = match opts.on_request {
            Some(f) => Some(make_tsfn(f)?),
            None => None,
        };
        let on_response = match opts.on_response {
            Some(f) => Some(make_tsfn(f)?),
            None => None,
        };
        let on_shutdown = match opts.on_shutdown {
            Some(f) => Some(make_tsfn(f)?),
            None => None,
        };

        Ok(Self {
            inner: Arc::new(JsFairing {
                name,
                kind,
                on_start,
                on_liftoff,
                on_request,
                on_response,
                on_shutdown,
                build_slot: Arc::new(Mutex::new(None)),
            }),
        })
    }

    /// Attach this fairing to an `Service` (the NAPI mirror of a `Service` instance).
    /// Mirrors `service.attach(fairing)`. Call before `app.launch()`.
    #[napi]
    pub fn attach(&self, app: &Service) -> napi::Result<()> {
        app.attach(self)
    }

    /// Register this fairing on the global registry consumed by `serve()`.
    #[napi]
    pub fn register(&self) -> napi::Result<()> {
        register_fairing(self)
    }
}

// ---------------------------------------------------------------------------
// service::fairing::Fairing implementation
// ---------------------------------------------------------------------------

#[service::async_trait]
impl ServiceFairing for JsFairing {
    fn info(&self) -> Info {
        Info {
            // Leak the name; `Info.name` is `&'static str` and fairings live for
            // the whole application lifetime. One small leak per fairing.
            name: Box::leak(self.name.clone().into_boxed_str()),
            kind: self.kind,
        }
    }

    async fn on_start(&self, service: Svc<Build>) -> FairingResult {
        match &self.on_start {
            Some(tsfn) => {
                // Stash the service in our shared slot, hand JS a live handle, then
                // take the (possibly mutated) service back and return it.
                *self.build_slot.lock().unwrap() = Some(service);
                let handle = ServiceHandle {
                    inner: self.build_slot.clone(),
                    shutdown: Arc::new(Mutex::new(None)),
                };
                let call = tsfn.call_async(handle).await;
                let service = self
                    .build_slot
                    .lock()
                    .unwrap()
                    .take()
                    .expect("fairing build slot lost");
                match call {
                    Ok(promise) => {
                        let _ = promise.await;
                        Ok(service)
                    }
                    // A failed JS callback aborts ignition by returning the service
                    // in the `Err` position (Service records the failure and aborts
                    // launch).
                    Err(_) => Err(service),
                }
            }
            None => Ok(service),
        }
    }

    async fn on_liftoff(&self, service: &Svc<Active>) {
        if let Some(tsfn) = &self.on_liftoff {
            let handle = ShutdownHandle {
                inner: service.shutdown(),
            };
            let _ = tsfn.call_async(handle).await;
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, data: &mut Data<'_>) {
        if let Some(tsfn) = &self.on_request {
            let payload = fairing_request_from(req, data).await;
            let _ = tsfn.call_async(payload).await;
        }
    }

    async fn on_response<'r>(&self, _req: &'r Request<'_>, res: &mut ServiceResponse<'r>) {
        if let Some(tsfn) = &self.on_response {
            let payload = fairing_response_from(res);
            if let Ok(promise) = tsfn.call_async(payload).await {
                if let Ok(fr) = promise.await {
                    apply_response(res, fr);
                }
            }
        }
    }

    async fn on_shutdown(&self, service: &Svc<Active>) {
        if let Some(tsfn) = &self.on_shutdown {
            let handle = ShutdownHandle {
                inner: service.shutdown(),
            };
            let _ = tsfn.call_async(handle).await;
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Convert the JS `FairingKind` object into Service's `Kind` bitset.
fn to_service_kind(k: &FairingKind) -> ServiceKind {
    // `Kind` exposes no public `empty()`/`EMPTY`; a zeroed `Kind` (a `usize`
    // newtype) is a valid empty bitset.
    let mut kind: ServiceKind = unsafe { std::mem::zeroed() };
    if k.start.unwrap_or(false) {
        kind = kind | ServiceKind::Ready;
    }
    if k.liftoff.unwrap_or(false) {
        kind = kind | ServiceKind::Liftoff;
    }
    if k.request.unwrap_or(false) {
        kind = kind | ServiceKind::Request;
    }
    if k.response.unwrap_or(false) {
        kind = kind | ServiceKind::Response;
    }
    if k.shutdown.unwrap_or(false) {
        kind = kind | ServiceKind::Shutdown;
    }
    if k.singleton.unwrap_or(false) {
        kind = kind | ServiceKind::Singleton;
    }
    kind
}

/// Build a read-only request snapshot. The body is peeked (non-consuming) so the
/// real data stream remains available to routing and handlers.
async fn fairing_request_from(req: &Request<'_>, data: &mut Data<'_>) -> FairingRequest {
    let method = req.method().as_str().to_string();
    let uri = req.uri().to_string();
    let headers: Vec<JsHeader> = req
        .headers()
        .iter()
        .map(|h| JsHeader {
            name: h.name.as_str().to_string(),
            value: h.value.to_string(),
        })
        .collect();
    let params: HashMap<String, String> = req
        .route()
        .map(|route| extract_params(&route.uri.to_string(), req.uri().path().as_str()))
        .unwrap_or_default();

    let body = {
        let peeked = data.peek(1 << 20).await;
        if peeked.is_empty() {
            None
        } else {
            Some(Buffer::from(peeked.to_vec()))
        }
    };

    FairingRequest {
        method,
        uri,
        headers,
        params: if params.is_empty() { None } else { Some(params) },
        body,
    }
}

/// Build a mutable response snapshot from a `service::Response`.
fn fairing_response_from(res: &ServiceResponse<'_>) -> FairingResponse {
    let status = res.status().code as u32;
    let headers: Vec<JsHeader> = res
        .headers()
        .iter()
        .map(|h| JsHeader {
            name: h.name.as_str().to_string(),
            value: h.value.to_string(),
        })
        .collect();
    FairingResponse {
        status,
        headers: Some(headers),
        body: None,
    }
}

/// Write a (mutated) response snapshot back onto the real `service::Response`.
fn apply_response(res: &mut ServiceResponse<'_>, fr: FairingResponse) {
    res.set_status(ServiceStatus::from_code(fr.status as u16).unwrap_or(ServiceStatus::Ok));
    if let Some(headers) = fr.headers {
        for h in headers {
            // Owned `String`s so the resulting `Cow` outlives the local `h`.
            res.set_raw_header(h.name, h.value);
        }
    }
    if let Some(body) = fr.body {
        let bytes = body.to_vec();
        res.set_sized_body(bytes.len(), Cursor::new(bytes));
    }
}

/// Align a Service route pattern's `<param>` segments with the actual request path
/// segments to populate `req.params`. Mirrors the helper in `service.rs`.
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

/// Wrap a JS function as a threadsafe function that receives `T` and resolves a
/// `Promise<R>` (e.g. `Promise<()>` or `Promise<FairingResponse>`). `T` must be a
/// NAPI value (all our handle/payload types are `#[napi]`-derived and thus
/// `ToNapiValue`); `R` is the JS-returned promise value.
fn make_tsfn<T, R>(f: Unknown) -> napi::Result<Arc<ThreadsafeFunction<T, R, T, Status, false, false, 0>>>
where
    T: ToNapiValue,
    R: FromNapiValue,
{
    let js = to_js_function(f);
    let tsfn = js
        .create_threadsafe_function::<T, T, R, Status, _, false, false, 0>(|ctx| Ok(ctx.value))
        .map_err(|e| Error::new(Status::GenericFailure, format!("invalid fairing callback: {e}")))?;
    Ok(Arc::new(tsfn))
}

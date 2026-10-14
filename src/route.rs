//! # NAPI bindings for Rocket's [`Route`] (`rocket::Route`).
//!
//! Exposes Rocket's `Route` structure, rank calculations, URI formatting,
//! media type formatting, and route construction methods to TypeScript.

use std::borrow::Cow;
use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
#[allow(deprecated)]
use napi::{JsFunction, Unknown};
use napi_derive::napi;

use rocket::http::{MediaType, Method, Status as RocketStatus};
use rocket::request::Request;
use rocket::route::{Handler, Outcome as RouteOutcome, Route as RocketRoute};
use rocket::Data;

use crate::server::{build_request, build_response, RequestInput, Response};

type HandlerTsfn = ThreadsafeFunction<
    RequestInput,
    Either<Response, Promise<Response>>,
    RequestInput,
    Status,
    false,
    false,
    0,
>;

#[allow(deprecated)]
fn to_js_function(handler: Unknown) -> JsFunction {
    debug_assert_eq!(
        std::mem::size_of::<Unknown>(),
        std::mem::size_of::<JsFunction>()
    );
    unsafe { std::mem::transmute_copy::<Unknown, JsFunction>(&handler) }
}

#[derive(Clone)]
struct NapiRouteHandler {
    tsfn: Arc<HandlerTsfn>,
}

#[rocket::async_trait]
impl Handler for NapiRouteHandler {
    async fn handle<'r>(&self, request: &'r Request<'_>, data: Data<'r>) -> RouteOutcome<'r> {
        let pattern = request
            .route()
            .map(|r| r.uri.to_string())
            .unwrap_or_default();
        let input = build_request(request, data, &pattern).await;
        let output = match self.tsfn.call_async(input).await {
            Ok(Either::A(payload)) => payload,
            Ok(Either::B(promise)) => match promise.await {
                Ok(payload) => payload,
                Err(_) => return RouteOutcome::Error(RocketStatus::InternalServerError),
            },
            Err(_) => return RouteOutcome::Error(RocketStatus::InternalServerError),
        };
        RouteOutcome::Success(build_response(&output))
    }
}

/// A request handling route, mirroring Rocket's `rocket::Route`.
#[napi]
pub struct Route {
    pub(crate) inner: RocketRoute,
}

#[napi]
impl Route {
    /// Create a new route with method, URI, and JS handler function.
    /// Default rank is computed automatically based on static/dynamic URI components.
    #[napi(constructor)]
    #[allow(deprecated)]
    pub fn new(method: String, uri: String, handler: Unknown) -> napi::Result<Self> {
        Self::ranked(None, method, uri, handler)
    }

    /// Create a new route with an explicit rank (or computed rank if `rank` is null/undefined).
    #[napi]
    #[allow(deprecated)]
    pub fn ranked(
        rank: Option<i64>,
        method: String,
        uri: String,
        handler: Unknown,
    ) -> napi::Result<Self> {
        let m: Method = method.parse().map_err(|_| {
            napi::Error::new(Status::InvalidArg, format!("invalid HTTP method: {method}"))
        })?;
        let f = to_js_function(handler);
        let tsfn: HandlerTsfn = f
            .create_threadsafe_function::<RequestInput, RequestInput, Either<Response, Promise<Response>>, Status, _, false, false, 0>(
                |ctx| Ok(ctx.value),
            )?;

        let route_handler = NapiRouteHandler {
            tsfn: Arc::new(tsfn),
        };

        let rank_isize = rank.map(|r| r as isize);
        let inner = RocketRoute::ranked(rank_isize, m, &uri, route_handler);

        Ok(Self { inner })
    }

    // -- getters / setters ------------------------------------------------

    #[napi(getter)]
    pub fn name(&self) -> Option<String> {
        self.inner.name.as_ref().map(|n| n.to_string())
    }

    #[napi(setter)]
    pub fn set_name(&mut self, name: Option<String>) {
        self.inner.name = name.map(Cow::Owned);
    }

    #[napi(getter)]
    pub fn method(&self) -> String {
        self.inner.method.as_str().to_string()
    }

    #[napi(getter)]
    pub fn uri(&self) -> String {
        self.inner.uri.to_string()
    }

    #[napi(getter)]
    pub fn rank(&self) -> i64 {
        self.inner.rank as i64
    }

    #[napi(setter)]
    pub fn set_rank(&mut self, rank: i64) {
        self.inner.rank = rank as isize;
    }

    #[napi(getter)]
    pub fn format(&self) -> Option<String> {
        self.inner.format.as_ref().map(|f| f.to_string())
    }

    #[napi(setter)]
    pub fn set_format(&mut self, format: Option<String>) -> napi::Result<()> {
        if let Some(ref fmt_str) = format {
            let mt = MediaType::parse_flexible(fmt_str).ok_or_else(|| {
                napi::Error::new(Status::InvalidArg, format!("invalid format/media type: {fmt_str}"))
            })?;
            self.inner.format = Some(mt);
        } else {
            self.inner.format = None;
        }
        Ok(())
    }

    /// Map the base mount point of this route using a base string.
    #[napi]
    pub fn map_base(&mut self, base: String) -> napi::Result<()> {
        let route = self.inner.clone();
        let mapped = route
            .map_base(|_old_base| base)
            .map_err(|e| napi::Error::new(Status::GenericFailure, format!("invalid base URI: {e}")))?;
        self.inner = mapped;
        Ok(())
    }
}

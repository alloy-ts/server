//! NAPI bindings for Rocket's [`Handler`] trait (`rocket::route::Handler`).
//!
//! This module allows JS callers to register and invoke custom request handlers
//! backed by a thread-safe JS function, bridging Rocket's `Handler` trait to
//! Node.js / TypeScript async closures.

use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
#[allow(deprecated)]
use napi::{JsFunction, Unknown};
use napi_derive::napi;

use rocket::data::Data;
use rocket::http::Status as RocketStatus;
use rocket::request::Request;
use rocket::route::{Handler as RocketHandler, Outcome as RouteOutcome};

use crate::server::{build_request, build_response, RequestInput, Response};

type HandlerTsfn =
    ThreadsafeFunction<RequestInput, Promise<Response>, RequestInput, Status, false, false, 0>;

#[allow(deprecated)]
fn to_js_function(handler: Unknown) -> JsFunction {
    debug_assert_eq!(
        std::mem::size_of::<Unknown>(),
        std::mem::size_of::<JsFunction>()
    );
    unsafe { std::mem::transmute_copy::<Unknown, JsFunction>(&handler) }
}

/// NAPI-wrapped custom route handler bridging Rocket's `Handler` trait to JS callbacks.
#[derive(Clone)]
pub struct NapiHandler {
    tsfn: Arc<HandlerTsfn>,
}

#[rocket::async_trait]
impl RocketHandler for NapiHandler {
    async fn handle<'r>(&self, req: &'r Request<'_>, data: Data<'r>) -> RouteOutcome<'r> {
        let pattern = req.route().map(|r| r.uri.to_string()).unwrap_or_default();
        let input = build_request(req, data, &pattern).await;
        let output = match self.tsfn.call_async(input).await {
            Ok(promise) => match promise.await {
                Ok(payload) => payload,
                Err(_) => return RouteOutcome::Error(RocketStatus::InternalServerError),
            },
            Err(_) => return RouteOutcome::Error(RocketStatus::InternalServerError),
        };
        RouteOutcome::Success(build_response(&output))
    }
}

/// NAPI class representing a Rocket route handler constructed from a JS function.
#[napi]
pub struct Handler {
    pub(crate) inner: NapiHandler,
}

#[napi]
impl Handler {
    /// Create a new custom Rocket handler from an async JS callback `(req: RequestInput) => Promise<Response>`.
    #[napi(constructor)]
    pub fn new(callback: Unknown<'_>) -> napi::Result<Self> {
        let f = to_js_function(callback);
        let tsfn: HandlerTsfn =
            f.create_threadsafe_function::<RequestInput, RequestInput, Promise<Response>, Status, _, false, false, 0>(
                |ctx| Ok(ctx.value),
            )?;
        Ok(Self {
            inner: NapiHandler { tsfn: Arc::new(tsfn) },
        })
    }
}

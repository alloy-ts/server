//! NAPI bindings for Rocket's [`Handler`] trait (`rocket::route::Handler`).
//!
//! Bridges JavaScript callback functions into Rocket's `Handler` trait,
//! allowing custom JS request handlers to be instantiated as Rocket handlers
//! and converted into Rocket `Route` objects via `toRoute(method, path)`.

use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
#[allow(deprecated)]
use napi::{JsFunction, Unknown};
use napi_derive::napi;

use rocket::http::Status as RocketStatus;
use rocket::request::Request;
use rocket::route::{Handler as RocketHandler, Outcome as RouteOutcome, Route as RocketRoute};
use rocket::Data;

use crate::route::Route;
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

/// A NAPI wrapper around a JS callback implementing Rocket's [`Handler`] trait.
#[napi]
#[derive(Clone)]
pub struct Handler {
    tsfn: Arc<HandlerTsfn>,
}

#[napi]
impl Handler {
    /// Create a new Rocket `Handler` wrapping a JavaScript callback function.
    #[napi(constructor)]
    pub fn new(callback: Unknown<'_>) -> napi::Result<Self> {
        let f = to_js_function(callback);
        let tsfn: HandlerTsfn =
            f.create_threadsafe_function::<RequestInput, RequestInput, Promise<Response>, Status, _, false, false, 0>(
                |ctx| Ok(ctx.value),
            )?;
        Ok(Self {
            tsfn: Arc::new(tsfn),
        })
    }

    /// Convert this handler into a Rocket `Route` given an HTTP method and path.
    #[napi]
    pub fn to_route(&self, method: String, path: String) -> napi::Result<Route> {
        let m: rocket::http::Method = method
            .parse()
            .map_err(|_| napi::Error::new(Status::InvalidArg, format!("invalid method: {method}")))?;
        let rocket_route = RocketRoute::new(m, &path, self.clone());
        Ok(Route::from_rocket(rocket_route))
    }
}

#[rocket::async_trait]
impl RocketHandler for Handler {
    async fn handle<'r>(&self, request: &'r Request<'_>, data: Data<'r>) -> RouteOutcome<'r> {
        let pattern = request
            .route()
            .map(|r| r.uri.to_string())
            .unwrap_or_default();

        let input = build_request(request, data, &pattern).await;
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

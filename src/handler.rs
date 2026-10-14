//! NAPI bindings for Rocket's [`Handler`](rocket::route::Handler) and [`Outcome`](rocket::route::Outcome).

use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
use napi::bindgen_prelude::Function;
#[allow(deprecated)]
use napi::{JsFunction, Unknown};
use napi_derive::napi;

use rocket::http::Status as RocketStatus;
use rocket::request::Request;
use rocket::route::{Handler as RocketRouteHandler, Outcome as RocketRouteOutcome};
use rocket::Data;

use crate::server::{build_request, build_response, RequestInput, Response};

#[allow(deprecated)]
fn to_js_function(handler: Unknown) -> JsFunction {
    debug_assert_eq!(
        std::mem::size_of::<Unknown>(),
        std::mem::size_of::<JsFunction>()
    );
    unsafe { std::mem::transmute_copy(&handler) }
}

/// Representation of Rocket's handler result handed back from a JS handler.
#[napi(object)]
#[derive(Clone, Debug)]
pub struct HandlerOutcome {
    /// `"success"`, `"error"`, or `"forward"`.
    pub kind: String,
    pub response: Option<Response>,
    pub status: Option<u32>,
}

/// Creates a `HandlerOutcome` indicating a successful response.
#[napi]
pub fn outcome_success(response: Response) -> HandlerOutcome {
    HandlerOutcome {
        kind: "success".to_string(),
        response: Some(response),
        status: None,
    }
}

/// Creates a `HandlerOutcome` indicating a failure with the given HTTP status.
#[napi]
pub fn outcome_error(status: u32) -> HandlerOutcome {
    HandlerOutcome {
        kind: "error".to_string(),
        response: None,
        status: Some(status),
    }
}

/// Creates a `HandlerOutcome` indicating Rocket should forward to the next matching route.
#[napi]
pub fn outcome_forward(status: Option<u32>) -> HandlerOutcome {
    HandlerOutcome {
        kind: "forward".to_string(),
        response: None,
        status: Some(status.unwrap_or(404)),
    }
}

type HandlerTsfn =
    ThreadsafeFunction<RequestInput, Promise<HandlerOutcome>, RequestInput, Status, false, false, 0>;

/// Bridge a JS callback function into Rocket's `Handler` trait.
#[napi]
#[derive(Clone)]
pub struct CustomHandler {
    #[napi(skip)]
    pub tsfn: Arc<HandlerTsfn>,
}

#[napi]
impl CustomHandler {
    #[napi(constructor)]
    #[allow(deprecated)]
    pub fn new(env: Env, handler: Unknown) -> napi::Result<Self> {
        let wrap_creator: Function = env.run_script("(fn) => async (...args) => await fn(...args)")?;
        let wrapped: Unknown = wrap_creator.call(handler)?;
        let f = to_js_function(wrapped);
        let tsfn: HandlerTsfn =
            f.create_threadsafe_function::<RequestInput, RequestInput, Promise<HandlerOutcome>, Status, _, false, false, 0>(
                |ctx| Ok(ctx.value),
            )?;
        Ok(CustomHandler {
            tsfn: Arc::new(tsfn),
        })
    }
}

#[rocket::async_trait]
impl RocketRouteHandler for CustomHandler {
    async fn handle<'r>(&self, request: &'r Request<'_>, mut data: Data<'r>) -> RocketRouteOutcome<'r> {
        let pattern = request
            .route()
            .map(|r| r.uri.to_string())
            .unwrap_or_default();
        let input = build_request(request, &mut data, &pattern).await;

        match self.tsfn.call_async(input).await {
            Ok(promise) => match promise.await {
                Ok(outcome) => match outcome.kind.as_str() {
                    "success" => {
                        if let Some(resp) = outcome.response {
                            RocketRouteOutcome::Success(build_response(&resp))
                        } else {
                            RocketRouteOutcome::Error(RocketStatus::InternalServerError)
                        }
                    }
                    "forward" => {
                        let code = outcome.status.unwrap_or(404) as u16;
                        let status = RocketStatus::from_code(code).unwrap_or(RocketStatus::NotFound);
                        RocketRouteOutcome::Forward((data, status))
                    }
                    "error" => {
                        let code = outcome.status.unwrap_or(500) as u16;
                        let status =
                            RocketStatus::from_code(code).unwrap_or(RocketStatus::InternalServerError);
                        RocketRouteOutcome::Error(status)
                    }
                    _ => RocketRouteOutcome::Error(RocketStatus::InternalServerError),
                },
                Err(_) => RocketRouteOutcome::Error(RocketStatus::InternalServerError),
            },
            Err(_) => RocketRouteOutcome::Error(RocketStatus::InternalServerError),
        }
    }
}

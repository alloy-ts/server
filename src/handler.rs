//! NAPI bindings for Rocket's [`Handler`](rocket::route::Handler) trait.
//!
//! Rocket's `Handler` trait enables custom request handling logic where request
//! handling can return one of three outcomes:
//!
//! 1. `Success(Response)` - Produce a response directly.
//! 2. `Error(Status)` - Forward to an error catcher for the given status code.
//! 3. `Forward(Status)` - Forward the request to the next matching route.
//!
//! This module exposes `HandlerBridge` to JavaScript, allowing JS handlers to
//! express full Rocket `Outcome` semantics (`success`, `error`, `forward`).

use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
#[allow(deprecated)]
use napi::{JsFunction, Unknown};
use napi_derive::napi;

use rocket::http::Status as RocketStatus;
use rocket::request::Request;
use rocket::response::Response as RocketResponse;
use rocket::catcher::{Handler as RocketCatcherHandler, Result as CatcherResult};
use rocket::route::{Handler as RocketRouteHandler, Outcome as RouteOutcome};
use rocket::Data;

use crate::main_rs::{build_request_payload, build_response, RequestPayload, ResponsePayload};

/// Wire outcome returned from a JS handler bridge.
#[napi(object)]
pub struct OutcomePayload {
    /// `"success"`, `"error"`, or `"forward"`.
    pub action: String,
    /// Response payload when `action` is `"success"`.
    pub response: Option<ResponsePayload>,
    /// Status code when `action` is `"error"` or `"forward"`.
    pub status: Option<u32>,
}

type JsHandlerTsfn =
    ThreadsafeFunction<(RequestPayload,), OutcomePayload, (RequestPayload,), Status, false, false, 0>;

#[allow(deprecated)]
fn to_js_function(handler: Unknown) -> JsFunction {
    debug_assert_eq!(
        std::mem::size_of::<Unknown>(),
        std::mem::size_of::<JsFunction>()
    );
    unsafe { std::mem::transmute_copy::<Unknown, JsFunction>(&handler) }
}

/// NAPI wrapper bridging Rocket's `Handler` trait to a JS callback.
#[napi]
#[derive(Clone)]
pub struct HandlerBridge {
    tsfn: Arc<JsHandlerTsfn>,
}

#[napi]
impl HandlerBridge {
    /// Create a new custom `HandlerBridge` from a JS function:
    /// `(req: RequestPayload) => OutcomePayload | Promise<OutcomePayload>`.
    #[napi(constructor)]
    #[allow(deprecated)]
    pub fn new(handler: Unknown) -> Result<Self> {
        let f = to_js_function(handler);
        let tsfn: JsHandlerTsfn =
            f.create_threadsafe_function(|ctx| Ok(ctx.value))?;
        Ok(Self {
            tsfn: Arc::new(tsfn),
        })
    }

    /// Convenience static helper returning a success outcome payload.
    #[napi]
    pub fn success(response: ResponsePayload) -> OutcomePayload {
        OutcomePayload {
            action: "success".to_string(),
            response: Some(response),
            status: None,
        }
    }

    /// Convenience static helper returning an error outcome payload.
    #[napi]
    pub fn error(status: u32) -> OutcomePayload {
        OutcomePayload {
            action: "error".to_string(),
            response: None,
            status: Some(status),
        }
    }

    /// Convenience static helper returning a forward outcome payload.
    #[napi]
    pub fn forward(status: u32) -> OutcomePayload {
        OutcomePayload {
            action: "forward".to_string(),
            response: None,
            status: Some(status),
        }
    }
}

#[rocket::async_trait]
impl RocketCatcherHandler for HandlerBridge {
    async fn handle<'r>(&self, status: RocketStatus, req: &'r Request<'_>) -> CatcherResult<'r> {
        let payload = build_request_payload(req, None).await;
        match self.tsfn.call_async((payload,)).await {
            Ok(outcome) => match outcome.action.as_str() {
                "success" => {
                    let resp = outcome
                        .response
                        .map(build_response)
                        .unwrap_or_else(|| RocketResponse::build().finalize());
                    Ok(resp)
                }
                "error" => {
                    let err_status = outcome
                        .status
                        .and_then(|s| RocketStatus::from_code(s as u16))
                        .unwrap_or(status);
                    Err(err_status)
                }
                _ => Err(status),
            },
            Err(_) => Err(status),
        }
    }
}

#[rocket::async_trait]
impl RocketRouteHandler for HandlerBridge {
    async fn handle<'r>(&self, req: &'r Request<'_>, data: Data<'r>) -> RouteOutcome<'r> {
        let payload = build_request_payload(req, None).await;
        match self.tsfn.call_async((payload,)).await {
            Ok(outcome) => match outcome.action.as_str() {
                "success" => {
                    let resp = outcome
                        .response
                        .map(build_response)
                        .unwrap_or_else(|| RocketResponse::build().finalize());
                    RouteOutcome::Success(resp)
                }
                "error" => {
                    let status = outcome
                        .status
                        .and_then(|s| RocketStatus::from_code(s as u16))
                        .unwrap_or(RocketStatus::InternalServerError);
                    RouteOutcome::Error(status)
                }
                "forward" => {
                    let status = outcome
                        .status
                        .and_then(|s| RocketStatus::from_code(s as u16))
                        .unwrap_or(RocketStatus::NotFound);
                    RouteOutcome::Forward((data, status))
                }
                _ => RouteOutcome::Error(RocketStatus::InternalServerError),
            },
            Err(_) => RouteOutcome::Error(RocketStatus::InternalServerError),
        }
    }
}

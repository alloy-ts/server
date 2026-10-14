//! NAPI bindings for Rocket's [`Handler`] trait and routing [`Outcome`].
//!
//! Rocket handlers process requests and return an [`Outcome`]:
//! - `Success(Response)`: Return a successful response to the client.
//! - `Error(u32)`: Invoke the error catcher for the given HTTP status code.
//! - `Forward`: Forward request to the next matching route in rank order.

#![allow(dead_code)]

use std::sync::Arc;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
#[allow(deprecated)]
use napi::{JsFunction, Unknown, ValueType};
use napi_derive::napi;
use rocket::http::Status as RocketStatus;
use rocket::request::Request;
use rocket::route::{Handler as RocketHandler, Outcome as RouteOutcome};
use rocket::Data;

use crate::server::{build_request, build_response, RequestInput, Response};

/// The outcome of a JS route handler execution.
#[napi(string_enum)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandlerOutcomeKind {
    Success,
    Error,
    Forward,
}

/// Outcome structure returned by a TypeScript handler or created via helper functions.
#[napi(object)]
pub struct HandlerOutcome {
    pub kind: HandlerOutcomeKind,
    pub response: Option<Response>,
    pub status: Option<u32>,
}

#[napi]
pub fn outcome_success(res: Response) -> HandlerOutcome {
    HandlerOutcome {
        kind: HandlerOutcomeKind::Success,
        response: Some(res),
        status: None,
    }
}

#[napi]
pub fn outcome_error(status: u32) -> HandlerOutcome {
    HandlerOutcome {
        kind: HandlerOutcomeKind::Error,
        response: None,
        status: Some(status),
    }
}

#[napi]
pub fn outcome_forward() -> HandlerOutcome {
    HandlerOutcome {
        kind: HandlerOutcomeKind::Forward,
        response: None,
        status: None,
    }
}

type HandlerTsfn = ThreadsafeFunction<RequestInput, Promise<HandlerOutcome>, RequestInput, Status, false, false, 0>;

#[allow(deprecated)]
fn to_js_function(handler: Unknown) -> napi::Result<JsFunction> {
    if handler.get_type()? != ValueType::Function {
        return Err(napi::Error::new(
            Status::InvalidArg,
            "expected a function for route handler",
        ));
    }
    debug_assert_eq!(
        std::mem::size_of::<Unknown>(),
        std::mem::size_of::<JsFunction>()
    );
    Ok(unsafe { std::mem::transmute_copy::<Unknown, JsFunction>(&handler) })
}

/// NAPI wrapper bridging a TypeScript handler callback into a [`rocket::route::Handler`].
#[derive(Clone)]
pub struct NapiJsHandler {
    tsfn: Arc<HandlerTsfn>,
}

impl NapiJsHandler {
    pub fn new(handler: Unknown) -> napi::Result<Self> {
        let f = to_js_function(handler)?;
        let tsfn: HandlerTsfn = f.create_threadsafe_function::<RequestInput, RequestInput, Promise<HandlerOutcome>, Status, _, false, false, 0>(
            |ctx| Ok(ctx.value),
        )?;
        Ok(Self {
            tsfn: Arc::new(tsfn),
        })
    }
}

#[rocket::async_trait]
impl RocketHandler for NapiJsHandler {
    async fn handle<'r>(&self, request: &'r Request<'_>, data: Data<'r>) -> RouteOutcome<'r> {
        let pattern = request
            .route()
            .map(|r| r.uri.to_string())
            .unwrap_or_default();

        let mut data = data;
        let input = build_request(request, &mut data, &pattern).await;

        let promise = match self.tsfn.call_async(input).await {
            Ok(p) => p,
            Err(_) => return RouteOutcome::Error(RocketStatus::InternalServerError),
        };

        let outcome = match promise.await {
            Ok(o) => o,
            Err(_) => return RouteOutcome::Error(RocketStatus::InternalServerError),
        };

        match outcome.kind {
            HandlerOutcomeKind::Success => {
                if let Some(resp) = outcome.response {
                    RouteOutcome::Success(build_response(&resp))
                } else {
                    RouteOutcome::Error(RocketStatus::InternalServerError)
                }
            }
            HandlerOutcomeKind::Error => {
                let code = outcome.status.unwrap_or(500) as u16;
                let status = RocketStatus::from_code(code).unwrap_or(RocketStatus::InternalServerError);
                RouteOutcome::Error(status)
            }
            HandlerOutcomeKind::Forward => RouteOutcome::Forward((data, RocketStatus::NotFound)),
        }
    }
}

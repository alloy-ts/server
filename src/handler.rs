//! NAPI bindings for Rocket's [`Handler`] and [`Outcome`] (`rocket::route::Handler`).
//!
//! Provides JS-friendly types and outcome constructors for request handlers.

use napi_derive::napi;

use crate::rocket_app::{JsHeader, ResponsePayload};

/// The outcome of handling a request in Rocket.
#[napi(string_enum)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutcomeKind {
    /// Request succeeded with a response.
    Success,
    /// Request failed with an HTTP status code.
    Error,
    /// Request should be forwarded to the next matching route.
    Forward,
}

/// Outcome returned by a custom JS route handler.
#[napi(object)]
#[derive(Clone, Debug)]
pub struct HandlerOutcome {
    pub kind: OutcomeKind,
    pub status: Option<u32>,
    pub response: Option<ResponsePayload>,
}

/// Construct a successful handler outcome with a response.
#[napi]
pub fn handler_outcome_success(status: Option<u32>, headers: Option<Vec<JsHeader>>, body: Option<Vec<u8>>) -> HandlerOutcome {
    HandlerOutcome {
        kind: OutcomeKind::Success,
        status: Some(status.unwrap_or(200)),
        response: Some(ResponsePayload {
            status: Some(status.unwrap_or(200)),
            headers,
            body,
        }),
    }
}

/// Construct an error handler outcome with an HTTP status code.
#[napi]
pub fn handler_outcome_error(status: u32) -> HandlerOutcome {
    HandlerOutcome {
        kind: OutcomeKind::Error,
        status: Some(status),
        response: None,
    }
}

/// Construct a forward handler outcome to pass request to next route.
#[napi]
pub fn handler_outcome_forward(status: Option<u32>) -> HandlerOutcome {
    HandlerOutcome {
        kind: OutcomeKind::Forward,
        status,
        response: None,
    }
}

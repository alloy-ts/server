//! Rocket Handler Outcome NAPI Bindings
//!
//! Provides JS bindings for Rocket's handler outcomes (`Outcome::Success`,
//! `Outcome::Error`, `Outcome::Forward`) and `HandlerOutcome` helper structures.

use napi_derive::napi;
use crate::server::Response;

/// The outcome of a Rocket route handler execution.
#[napi]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandlerOutcomeKind {
    /// Handler succeeded and returned a response.
    Success,
    /// Handler failed with an HTTP status code.
    Error,
    /// Handler forwarded the request (e.g. to the next matching route or catcher).
    Forward,
}

/// NAPI outcome object representing the result of calling a Rocket route handler.
#[napi(object)]
pub struct HandlerOutcome {
    /// Kind of outcome (`Success`, `Error`, `Forward`).
    pub kind: HandlerOutcomeKind,
    /// HTTP status code associated with the outcome.
    pub status: u32,
    /// Response payload produced if the outcome is `Success`.
    pub response: Option<Response>,
}

/// Create a successful handler outcome with the given response.
#[napi]
pub fn route_outcome_success(response: Response) -> HandlerOutcome {
    let status = response.status;
    HandlerOutcome {
        kind: HandlerOutcomeKind::Success,
        status,
        response: Some(response),
    }
}

/// Create an error handler outcome with the given HTTP status code.
#[napi]
pub fn route_outcome_error(status: u32) -> HandlerOutcome {
    HandlerOutcome {
        kind: HandlerOutcomeKind::Error,
        status,
        response: None,
    }
}

/// Create a forward handler outcome with an optional HTTP status code (defaults to 404).
#[napi]
pub fn route_outcome_forward(status: Option<u32>) -> HandlerOutcome {
    HandlerOutcome {
        kind: HandlerOutcomeKind::Forward,
        status: status.unwrap_or(404),
        response: None,
    }
}

//! NAPI bindings for Rocket's [`Handler`] and [`Outcome`] (`rocket::route::Handler`, `rocket::route::Outcome`).
//!
//! Rocket request handlers accept a request and body payload, returning an
//! `Outcome` (`Success`, `Error`, or `Forward`). This module provides NAPI types
//! representing custom handlers and outcomes for TypeScript.

use napi_derive::napi;
use crate::server::Response;

/// The outcome returned by a Rocket request handler (`rocket::route::Outcome`).
///
/// - `Success`: Handler successfully processed the request and produced a `Response`.
/// - `Error`: Handler failed with an HTTP status code (e.g. `500` or `400`).
/// - `Forward`: Handler forwarded request to next matching route with an optional status code.
#[derive(Clone, Debug)]
#[napi(object)]
pub struct HandlerOutcome {
    pub kind: String, // "success" | "error" | "forward"
    pub response: Option<Response>,
    pub status: Option<u32>,
}

/// Helper functions to construct `HandlerOutcome` objects.
#[napi]
pub fn create_success_outcome(response: Response) -> HandlerOutcome {
    HandlerOutcome {
        kind: "success".to_string(),
        response: Some(response),
        status: None,
    }
}

#[napi]
pub fn create_error_outcome(status: u32) -> HandlerOutcome {
    HandlerOutcome {
        kind: "error".to_string(),
        response: None,
        status: Some(status),
    }
}

#[napi]
pub fn create_forward_outcome(status: Option<u32>) -> HandlerOutcome {
    HandlerOutcome {
        kind: "forward".to_string(),
        response: None,
        status,
    }
}

//! NAPI bindings for Rocket's [`Handler`](rocket::route::Handler) and handler [`Outcome`].
//!
//! Exposes handler outcomes (`Success`, `Error`, `Forward`) and bridging
//! structures to allow JavaScript request handlers to integrate directly into
//! Rocket's request routing pipeline.

use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::server::Response;

/// Outcome returned by a custom Rocket handler.
#[napi(object)]
#[derive(Clone, Debug)]
pub struct HandlerOutcome {
    /// `"success"`, `"error"`, or `"forward"`.
    pub kind: String,
    /// HTTP status code for `"error"` or `"success"` outcomes.
    pub status: Option<u32>,
    /// Response payload when `kind == "success"`.
    pub response: Option<Response>,
}

/// Returns a Success outcome wrapping the given response payload.
#[napi]
pub fn handler_outcome_success(response: Response) -> HandlerOutcome {
    HandlerOutcome {
        kind: "success".to_string(),
        status: Some(response.status),
        response: Some(response),
    }
}

/// Returns an Error outcome for the given HTTP status code.
#[napi]
pub fn handler_outcome_error(status: u32) -> HandlerOutcome {
    HandlerOutcome {
        kind: "error".to_string(),
        status: Some(status),
        response: None,
    }
}

/// Returns a Forward outcome indicating Rocket should try the next route.
#[napi]
pub fn handler_outcome_forward() -> HandlerOutcome {
    HandlerOutcome {
        kind: "forward".to_string(),
        status: None,
        response: None,
    }
}

/// Helper function creating a text response payload for custom handlers.
#[napi]
pub fn text_response(text: String, status: Option<u32>) -> Response {
    let mut headers = std::collections::HashMap::new();
    headers.insert("Content-Type".to_string(), "text/plain; charset=utf-8".to_string());
    Response {
        status: status.unwrap_or(200),
        headers,
        body: Some(text.into_bytes()),
    }
}

/// Helper function creating a JSON response payload for custom handlers.
#[napi]
pub fn json_response(json: serde_json::Value, status: Option<u32>) -> napi::Result<Response> {
    let mut headers = std::collections::HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    let bytes = serde_json::to_vec(&json)
        .map_err(|e| napi::Error::new(Status::GenericFailure, e.to_string()))?;
    Ok(Response {
        status: status.unwrap_or(200),
        headers,
        body: Some(bytes),
    })
}

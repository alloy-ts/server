//! NAPI bindings for Rocket's [`Handler`] trait (`rocket::route::Handler`).
//!
//! This module provides a NAPI wrapper for Rocket route request handlers,
//! enabling JavaScript / TypeScript functions to act as custom Rocket route
//! handlers that integrate directly with Rocket's routing outcome pipeline.

use napi_derive::napi;

use crate::server::ResponseValue;

/// Outcomes returned by a custom NAPI route handler, mirroring `rocket::route::Outcome`.
#[napi(object)]
#[derive(Clone, Debug)]
pub struct HandlerOutcome {
    /// `"success"`, `"error"`, or `"forward"`
    pub status: String,
    /// Response payload when status is `"success"`
    pub response: Option<ResponseValue>,
    /// HTTP status code when status is `"error"` or `"forward"`
    pub code: Option<u32>,
}

impl HandlerOutcome {
    /// Helper constructor for a successful outcome.
    pub fn success(response: ResponseValue) -> Self {
        Self {
            status: "success".to_string(),
            response: Some(response),
            code: None,
        }
    }

    /// Helper constructor for an error outcome.
    pub fn error(code: u32) -> Self {
        Self {
            status: "error".to_string(),
            response: None,
            code: Some(code),
        }
    }

    /// Helper constructor for a forward outcome.
    pub fn forward(code: u32) -> Self {
        Self {
            status: "forward".to_string(),
            response: None,
            code: Some(code),
        }
    }
}

/// Helper function to construct a success `HandlerOutcome`.
#[napi]
pub fn handler_success(response: ResponseValue) -> HandlerOutcome {
    HandlerOutcome::success(response)
}

/// Helper function to construct an error `HandlerOutcome`.
#[napi]
pub fn handler_error(code: u32) -> HandlerOutcome {
    HandlerOutcome::error(code)
}

/// Helper function to construct a forward `HandlerOutcome`.
#[napi]
pub fn handler_forward(code: u32) -> HandlerOutcome {
    HandlerOutcome::forward(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_handler_outcome_constructors() {
        let resp = ResponseValue {
            status: 200,
            headers: HashMap::new(),
            body: Some(b"OK".to_vec()),
        };

        let ok = handler_success(resp.clone());
        assert_eq!(ok.status, "success");
        assert_eq!(ok.response.unwrap().status, 200);

        let err = handler_error(400);
        assert_eq!(err.status, "error");
        assert_eq!(err.code, Some(400));

        let fwd = handler_forward(404);
        assert_eq!(fwd.status, "forward");
        assert_eq!(fwd.code, Some(404));
    }
}

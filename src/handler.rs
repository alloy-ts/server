//! NAPI bindings for Rocket's [`Handler`](rocket::route::Handler) trait.
//!
//! Rocket's `Handler` handles an incoming [`Request`](rocket::Request) and
//! [`Data`](rocket::Data) body stream, returning a route [`Outcome`](rocket::route::Outcome):
//! - `Success(Response)`
//! - `Error(Status)`
//! - `Forward((Data, Status))`

use napi_derive::napi;

use crate::server::{Response, ResponseInit};

/// The outcome returned by a custom JS route handler.
#[napi(object)]
#[derive(Clone, Debug)]
pub struct HandlerOutcome {
    /// `"success"`, `"error"`, or `"forward"`.
    pub kind: String,
    /// Response payload (when `kind == "success"`).
    pub response: Option<Response>,
    /// HTTP status code (when `kind == "error"` or `kind == "forward"`).
    pub status: Option<u32>,
}

/// Create a successful handler outcome wrapping a [`Response`].
#[napi]
pub fn outcome_success(body: Option<String>, init: Option<ResponseInit>) -> HandlerOutcome {
    let resp = crate::server::response(body, init);
    HandlerOutcome {
        kind: "success".to_string(),
        response: Some(resp),
        status: None,
    }
}

/// Create an error handler outcome wrapping an HTTP status code.
#[napi]
pub fn outcome_error(status: u32) -> HandlerOutcome {
    HandlerOutcome {
        kind: "error".to_string(),
        response: None,
        status: Some(status),
    }
}

/// Create a forward handler outcome to pass request to the next matching route.
#[napi]
pub fn outcome_forward(status: Option<u32>) -> HandlerOutcome {
    HandlerOutcome {
        kind: "forward".to_string(),
        response: None,
        status: Some(status.unwrap_or(404)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outcome_helpers() {
        let success = outcome_success(Some("OK".to_string()), None);
        assert_eq!(success.kind, "success");
        assert_eq!(success.response.unwrap().status, 200);

        let error = outcome_error(400);
        assert_eq!(error.kind, "error");
        assert_eq!(error.status, Some(400));

        let forward = outcome_forward(Some(404));
        assert_eq!(forward.kind, "forward");
        assert_eq!(forward.status, Some(404));
    }
}

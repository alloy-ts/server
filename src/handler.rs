//! # NAPI bindings for Rocket [`Handler`] (`rocket::route::Handler`) and [`Outcome`].

use napi_derive::napi;
use crate::server::Response;

/// Outcome returned by a custom Rocket route request handler.
#[napi(object)]
#[derive(Clone, Debug)]
pub struct HandlerOutcome {
    /// Outcome kind: `"success"`, `"error"`, or `"forward"`.
    pub kind: String,
    /// Response payload (when `kind == "success"`).
    pub response: Option<Response>,
    /// HTTP status code (when `kind == "error"` or `"forward"`).
    pub status: Option<u32>,
}

#[napi]
impl HandlerOutcome {
    /// Return a `Success` outcome wrapping a response.
    #[napi]
    pub fn success(response: Response) -> Self {
        Self {
            kind: "success".to_string(),
            response: Some(response),
            status: None,
        }
    }

    /// Return an `Error` outcome wrapping an HTTP status code.
    #[napi]
    pub fn error(status: u32) -> Self {
        Self {
            kind: "error".to_string(),
            response: None,
            status: Some(status),
        }
    }

    /// Return a `Forward` outcome passing control to the next matching route.
    #[napi]
    pub fn forward(status: Option<u32>) -> Self {
        Self {
            kind: "forward".to_string(),
            response: None,
            status,
        }
    }
}

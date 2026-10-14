//! NAPI bindings for Rocket's [`Handler`](rocket::route::Handler) trait.
//!
//! Exposes a NAPI-friendly JS handler wrapper and outcome builder mirroring
//! Rocket's `Handler` and `Outcome`.

use napi_derive::napi;

use crate::server::ResponsePayload;

/// JS-facing representation of Rocket's route handling Outcome (`Success`,
/// `Error`, or `Forward`).
#[napi(object)]
pub struct HandlerOutcome {
    pub status: u32,
    pub is_forward: Option<bool>,
    pub response: Option<ResponsePayload>,
}

/// NAPI wrapper around route handlers.
#[napi]
pub struct Handler {
    name: Option<String>,
}

#[napi]
impl Handler {
    #[napi(constructor)]
    pub fn new(name: Option<String>) -> Self {
        Self { name }
    }

    #[napi(getter)]
    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }

    /// Build a success outcome wrapping a response.
    #[napi]
    pub fn success(response: ResponsePayload) -> HandlerOutcome {
        HandlerOutcome {
            status: response.status,
            is_forward: Some(false),
            response: Some(response),
        }
    }

    /// Build an error outcome with status code.
    #[napi]
    pub fn error(status: u32) -> HandlerOutcome {
        HandlerOutcome {
            status,
            is_forward: Some(false),
            response: None,
        }
    }

    /// Build a forward outcome to pass request to next route.
    #[napi]
    pub fn forward(status: Option<u32>) -> HandlerOutcome {
        HandlerOutcome {
            status: status.unwrap_or(404),
            is_forward: Some(true),
            response: None,
        }
    }
}

//! # NAPI bindings for Rocket's `Handler` trait concepts.
//!
//! Provides a wrapper for JS request handlers and outcome abstractions
//! matching `rocket::route::Handler` and `rocket::route::Outcome`.

use napi_derive::napi;

#[napi(object)]
pub struct HandlerOutcome {
    pub status: u32,
    pub body: Option<String>,
    pub forwarded: bool,
}

#[napi]
pub fn create_handler_outcome_success(body: String, status: Option<u32>) -> HandlerOutcome {
    HandlerOutcome {
        status: status.unwrap_or(200),
        body: Some(body),
        forwarded: false,
    }
}

#[napi]
pub fn create_handler_outcome_error(status: u32) -> HandlerOutcome {
    HandlerOutcome {
        status,
        body: None,
        forwarded: false,
    }
}

#[napi]
pub fn create_handler_outcome_forward() -> HandlerOutcome {
    HandlerOutcome {
        status: 404,
        body: None,
        forwarded: true,
    }
}

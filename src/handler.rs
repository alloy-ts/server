//! NAPI bindings for Rocket's [`Handler`](rocket::route::Handler) trait.

use napi_derive::napi;

/// Representation of a handler's outcome status.
#[napi]
pub enum HandlerOutcomeStatus {
    Success,
    Forward,
    Error,
}

/// Metadata describing a custom route handler.
#[napi(object)]
pub struct HandlerMetadata {
    pub name: Option<String>,
    pub status: HandlerOutcomeStatus,
}

/// Creates a new `HandlerMetadata` object for custom handler registration.
#[napi]
pub fn create_handler(
    name: Option<String>,
    status: Option<HandlerOutcomeStatus>,
) -> HandlerMetadata {
    HandlerMetadata {
        name,
        status: status.unwrap_or(HandlerOutcomeStatus::Success),
    }
}

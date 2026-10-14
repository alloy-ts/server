//! NAPI bindings for Rocket's [`Handler`](rocket::route::Handler) trait and [`Outcome`](rocket::route::Outcome).
//!
//! Exposes request handler outcomes (Success, Forward, Error) and handler metadata helpers.

use napi_derive::napi;

use crate::server::Response;

/// Outcome returned by a Rocket request handler: Success(Response), Forward(status), or Error(status).
#[napi(object)]
#[derive(Clone, Debug)]
pub struct HandlerOutcome {
    pub kind: String,
    pub response: Option<Response>,
    pub status: Option<u32>,
}

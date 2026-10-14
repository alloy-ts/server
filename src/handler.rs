//! NAPI bindings for Rocket's [`Handler`] trait and routing [`Outcome`].
//!
//! Exposes `Outcome` status variants and helpers to represent Success, Error,
//! and Forward route handling outcomes.

use napi_derive::napi;
use crate::server::Response;

/// Outcome variant kind for JS route handlers.
#[napi(string_enum)]
pub enum OutcomeKind {
    Success,
    Error,
    Forward,
}

/// NAPI representation of Rocket's `route::Outcome`.
#[napi(object)]
pub struct Outcome {
    pub kind: OutcomeKind,
    pub response: Option<Response>,
    pub status: Option<u32>,
}

/// Helper constructors for `Outcome`.
#[napi]
pub struct OutcomeHelper;

#[napi]
impl OutcomeHelper {
    #[napi]
    pub fn success(response: Response) -> Outcome {
        Outcome {
            kind: OutcomeKind::Success,
            response: Some(response),
            status: None,
        }
    }

    #[napi]
    pub fn error(status: u32) -> Outcome {
        Outcome {
            kind: OutcomeKind::Error,
            response: None,
            status: Some(status),
        }
    }

    #[napi]
    pub fn forward(status: u32) -> Outcome {
        Outcome {
            kind: OutcomeKind::Forward,
            response: None,
            status: Some(status),
        }
    }
}

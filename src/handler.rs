//! NAPI bindings for Rocket's [`Handler`] (`rocket::route::Handler`) and [`Outcome`].

use napi_derive::napi;

/// NAPI representation of Rocket's route handling `Outcome`.
#[napi(object)]
pub struct Outcome {
    pub kind: String,
    pub status: Option<u32>,
}

/// Helper methods for constructing Rocket route outcomes.
#[napi]
pub struct OutcomeHelper;

#[napi]
impl OutcomeHelper {
    /// Creates a Success outcome with status 200.
    #[napi]
    pub fn success() -> Outcome {
        Outcome {
            kind: "success".to_string(),
            status: Some(200),
        }
    }

    /// Creates an Error outcome with given HTTP status code.
    #[napi]
    pub fn error(code: u32) -> Outcome {
        Outcome {
            kind: "error".to_string(),
            status: Some(code),
        }
    }

    /// Creates a Forward outcome with given HTTP status code.
    #[napi]
    pub fn forward(code: u32) -> Outcome {
        Outcome {
            kind: "forward".to_string(),
            status: Some(code),
        }
    }
}

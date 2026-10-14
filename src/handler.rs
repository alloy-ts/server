//! # Rocket - Handler NAPI Bindings
//!
//! NAPI bindings for Rocket's `Handler` trait and handler outcomes (`Outcome`).

use napi::bindgen_prelude::*;
use napi_derive::napi;

/// Representation of a Rocket handler outcome status.
#[napi]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutcomeStatus {
  Success,
  Error,
  Forward,
}

/// NAPI structure representing the outcome of a Rocket request handler execution.
#[napi]
#[derive(Clone)]
pub struct Outcome {
  /// The outcome status (Success, Error, or Forward).
  pub status: OutcomeStatus,
  /// HTTP status code associated with the outcome (e.g. 200, 404, 500).
  pub code: u16,
  /// Response body content if successful or error response.
  pub body: Option<Vec<u8>>,
}

#[napi]
impl Outcome {
  #[napi(constructor)]
  pub fn new(status: OutcomeStatus, code: u16, body: Option<Buffer>) -> Self {
    Self {
      status,
      code,
      body: body.map(|b| b.to_vec()),
    }
  }

  /// Creates a successful Outcome with status code 200 and an optional body.
  #[napi]
  pub fn success(body: Option<Buffer>) -> Self {
    Self {
      status: OutcomeStatus::Success,
      code: 200,
      body: body.map(|b| b.to_vec()),
    }
  }

  /// Creates an Error Outcome with a specific HTTP status code.
  #[napi]
  pub fn error(code: u16) -> Self {
    Self {
      status: OutcomeStatus::Error,
      code,
      body: None,
    }
  }

  /// Creates a Forward Outcome to forward the request to the next matching route.
  #[napi]
  pub fn forward(code: u16) -> Self {
    Self {
      status: OutcomeStatus::Forward,
      code,
      body: None,
    }
  }

  /// Returns the body as a Buffer or null.
  #[napi(getter)]
  pub fn get_body(&self) -> Option<Buffer> {
    self.body.as_ref().map(|b| Buffer::from(b.clone()))
  }
}

/// A custom handler wrapper that can be passed from JavaScript/TypeScript.
#[napi]
pub struct Handler {
  handler_type: String,
}

#[napi]
impl Handler {
  #[napi(constructor)]
  pub fn new(handler_type: Option<String>) -> Self {
    Self {
      handler_type: handler_type.unwrap_or_else(|| "custom".to_string()),
    }
  }

  /// Returns the identifier or type of the custom handler.
  #[napi(getter)]
  pub fn handler_type(&self) -> String {
    self.handler_type.clone()
  }
}

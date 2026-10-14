//! # NAPI bindings for Rocket's [`Handler`] (`rocket::route::Handler`).
//!
//! Bridges JavaScript function callbacks to Rocket request handlers.

use napi_derive::napi;

#[napi]
pub struct Handler {
  pub name: Option<String>,
}

#[napi]
impl Handler {
  /// Creates a new named handler wrapper.
  #[napi(constructor)]
  pub fn new(name: Option<String>) -> Self {
    Handler { name }
  }
}

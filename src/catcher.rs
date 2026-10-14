//! # Rocket - Catcher NAPI Bindings
//!
//! NAPI bindings for Rocket's `Catcher` struct.

use napi::bindgen_prelude::*;
use napi_derive::napi;

/// An error catching route.
#[napi]
#[derive(Clone, Debug)]
pub struct Catcher {
  /// The name of this catcher, if one was given.
  pub name: Option<String>,
  /// The mount point.
  pub base: String,
  /// The HTTP status code to match against if this catcher is not default.
  pub code: Option<u32>,
}

#[napi]
impl Catcher {
  #[napi(constructor)]
  pub fn new(code: Option<u32>, base: Option<String>, name: Option<String>) -> napi::Result<Self> {
    if let Some(c) = code {
      if !(400..600).contains(&c) {
        return Err(Error::new(
          Status::InvalidArg,
          format!("status code must be in range [400, 600), got {}", c),
        ));
      }
    }

    let base = base.unwrap_or_else(|| "/".to_string());
    let norm_base = if base.starts_with('/') {
      base
    } else {
      format!("/{base}")
    };

    Ok(Catcher {
      name,
      base: norm_base,
      code,
    })
  }

  /// Returns true if this catcher is a default catcher (code is null).
  #[napi(getter)]
  pub fn is_default(&self) -> bool {
    self.code.is_none()
  }

  /// Maps the base mount point of this catcher.
  #[napi]
  pub fn map_base(&mut self, new_base: String) {
    let norm = if new_base.starts_with('/') {
      new_base
    } else {
      format!("/{new_base}")
    };
    self.base = norm;
  }
}

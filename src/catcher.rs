//! NAPI bindings for Rocket's [`Catcher`] (`rocket::catcher::Catcher`).
//!
//! Exposes `Catcher` as a NAPI class with constructor, property getters
//! (`name`, `base`, `code`), static factory method (`default_catcher`), and methods (`map_base`, `to_string`).

use napi::bindgen_prelude::Error as NapiError;
use napi::Status;
use napi_derive::napi;

use rocket::catcher::Catcher as RocketCatcher;

use crate::handler::Handler;

/// An error-catching Rocket route NAPI wrapper.
#[napi]
#[derive(Clone)]
pub struct Catcher {
    pub name: Option<String>,
    pub base: String,
    pub code: Option<u32>,
    pub(crate) inner: RocketCatcher,
}

#[napi]
impl Catcher {
    /// Create a new `Catcher` for the given HTTP error status code (400-599),
    /// or a default catcher if code is `None`/`null`/`undefined`.
    #[napi(constructor)]
    pub fn new(code: Option<u32>, handler: &Handler) -> napi::Result<Self> {
        let code_u16 = match code {
            Some(c) => {
                if !(400..600).contains(&c) {
                    return Err(NapiError::new(
                        Status::InvalidArg,
                        format!("status code `{c}` outside HTTP error range [400, 600)"),
                    ));
                }
                Some(c as u16)
            }
            None => None,
        };

        let catcher = RocketCatcher::new(code_u16, handler.clone());
        Ok(Self::from_rocket(catcher))
    }

    /// Create a Rocket default catcher.
    #[napi(factory)]
    pub fn default_catcher() -> Self {
        Self::from_rocket(RocketCatcher::default())
    }

    /// Convert a concrete `rocket::catcher::Catcher` into a NAPI `Catcher` wrapper.
    pub(crate) fn from_rocket(catcher: RocketCatcher) -> Self {
        let name = catcher.name.as_ref().map(|n| n.to_string());
        let base = catcher.base.path().to_string();
        let code = catcher.code.map(|c| c as u32);

        Self {
            name,
            base,
            code,
            inner: catcher,
        }
    }

    /// Map the base mount point of this catcher, returning a new `Catcher`.
    #[napi]
    pub fn map_base(&self, base: String) -> napi::Result<Self> {
        let catcher = self.inner.clone();
        let mapped = catcher.map_base(|_| base).map_err(|e| {
            NapiError::new(Status::InvalidArg, format!("invalid base URI: {e}"))
        })?;
        Ok(Self::from_rocket(mapped))
    }

    /// Return string representation of the catcher.
    #[napi]
    pub fn to_string(&self) -> String {
        self.inner.to_string()
    }

    #[napi(getter)]
    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }

    #[napi(getter)]
    pub fn base(&self) -> String {
        self.base.clone()
    }

    #[napi(getter)]
    pub fn code(&self) -> Option<u32> {
        self.code
    }
}

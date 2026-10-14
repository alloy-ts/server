//! # Rocket - Catcher NAPI Bindings
//!
//! Exposes Rocket's `Catcher` structure to TypeScript / Node.js.
//! Catchers handle error statuses (400-599) or default fallback errors.

use napi::bindgen_prelude::Error as NapiError;
use napi::Status;
use napi_derive::napi;
use rocket::http::uri::Origin;

/// An error catching route representation in NAPI.
#[napi]
#[derive(Clone, Debug)]
pub struct Catcher {
    /// The name of this catcher, if given.
    pub name: Option<String>,
    /// The mount point base path.
    pub base: String,
    /// The HTTP status code to match, or `None` if default catcher.
    pub code: Option<u16>,
}

#[napi]
impl Catcher {
    /// Creates a new Catcher with optional HTTP status code (400..600) and base URI.
    #[napi(constructor)]
    pub fn new(code: Option<u16>, base: Option<String>) -> napi::Result<Self> {
        if let Some(c) = code {
            if !(400..600).contains(&c) {
                return Err(NapiError::new(
                    Status::InvalidArg,
                    format!("status code {c} is not in HTTP error range [400, 600)"),
                ));
            }
        }

        let base_path = base.unwrap_or_else(|| "/".to_string());
        let origin = Origin::parse_owned(base_path.clone())
            .map_err(|e| NapiError::new(Status::InvalidArg, format!("invalid base URI: {e}")))?
            .into_normalized();

        Ok(Self {
            name: None,
            base: origin.path().to_string(),
            code,
        })
    }

    /// Creates a default catcher matching any unhandled status code.
    #[napi]
    pub fn default_catcher(base: Option<String>) -> napi::Result<Self> {
        let mut catcher = Self::new(None, base)?;
        catcher.name = Some("<Rocket Catcher>".to_string());
        Ok(catcher)
    }

    /// Maps the base URI of this catcher.
    #[napi]
    pub fn map_base(&mut self, new_base: String) -> napi::Result<()> {
        let origin = Origin::parse_owned(new_base)
            .map_err(|e| NapiError::new(Status::InvalidArg, format!("invalid base URI: {e}")))?
            .into_normalized();

        self.base = origin.path().to_string();
        Ok(())
    }

    /// String representation of the catcher.
    #[napi(js_name = "toString")]
    pub fn to_js_string(&self) -> String {
        let name_str = self
            .name
            .as_ref()
            .map(|n| format!("({n}) "))
            .unwrap_or_default();
        let code_str = self
            .code
            .map(|c| c.to_string())
            .unwrap_or_else(|| "default".to_string());

        format!("{name_str}{}{} ", self.base, code_str)
    }
}

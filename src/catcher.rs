//! NAPI bindings for Rocket's error catching route [`Catcher`] (`rocket::Catcher`).

use napi::bindgen_prelude::*;
use napi_derive::napi;

/// NAPI mirror of Rocket's error catching `Catcher`.
#[napi]
pub struct Catcher {
    pub name: Option<String>,
    pub base: String,
    pub code: Option<u16>,
}

#[napi]
impl Catcher {
    /// Creates a new Catcher for the given status code (or `None` for default).
    #[napi(constructor)]
    pub fn new(code: Option<u32>, name: Option<String>) -> Result<Self> {
        let code_u16 = if let Some(c) = code {
            if c < 400 || c >= 600 {
                return Err(Error::new(
                    Status::InvalidArg,
                    format!("Status code {c} out of valid error catcher range [400, 600)"),
                ));
            }
            Some(c as u16)
        } else {
            None
        };

        Ok(Catcher {
            name,
            base: "/".to_string(),
            code: code_u16,
        })
    }

    /// Maps the base URI of this catcher.
    #[napi]
    pub fn map_base(&mut self, new_base: String) {
        self.base = new_base;
    }
}

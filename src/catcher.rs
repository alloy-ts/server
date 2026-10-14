//! NAPI bindings for Rocket's [`Catcher`] (`rocket::catcher::Catcher`).
//!
//! An error catching route. Catchers run when errors are produced by the
//! application or guards/responders.

use napi_derive::napi;

/// NAPI mirror of Rocket's `Catcher` metadata.
#[napi]
pub struct Catcher {
    name: Option<String>,
    base: String,
    code: Option<u16>,
}

#[napi]
impl Catcher {
    /// Creates a catcher for the given HTTP status code, or a default catcher if code is null.
    #[napi(constructor)]
    pub fn new(code: Option<u16>, name: Option<String>) -> Self {
        if let Some(c) = code {
            assert!((400..600).contains(&c), "code must be in range [400, 600)");
        }
        Self {
            name,
            base: "/".to_string(),
            code,
        }
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
    pub fn code(&self) -> Option<u16> {
        self.code
    }

    #[napi]
    pub fn map_base(&mut self, base: String) {
        let clean = if base.starts_with('/') {
            base
        } else {
            format!("/{base}")
        };
        self.base = clean.trim_end_matches('/').to_string();
        if self.base.is_empty() {
            self.base = "/".to_string();
        }
    }
}

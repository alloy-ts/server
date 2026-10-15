//! NAPI bindings for Rocket's [`Catcher`] (`rocket::catcher::Catcher`).
//!
//! Catchers are routes that run when errors are produced by the application.
//! They consist of a name, base mount point, optional HTTP status code, and handler.

use napi_derive::napi;

/// NAPI mirror of Rocket's `Catcher`.
#[napi]
#[derive(Clone)]
pub struct Catcher {
    name: Option<String>,
    base: String,
    code: Option<u32>,
}

#[napi]
impl Catcher {
    /// Creates a new `Catcher` with status code (or `null` for default) and optional name/base.
    #[napi(constructor)]
    pub fn new(code: Option<u32>, name: Option<String>, base: Option<String>) -> Self {
        let base_clean = base.unwrap_or_else(|| "/".to_string());
        if let Some(c) = code {
            assert!((400..600).contains(&c), "status code must be in [400, 600)");
        }
        Catcher {
            name,
            base: base_clean,
            code,
        }
    }

    /// The name of this catcher, if given.
    #[napi(getter)]
    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }

    /// The base mount point.
    #[napi(getter)]
    pub fn base(&self) -> String {
        self.base.clone()
    }

    /// The HTTP status code to match against (or `null` if default).
    #[napi(getter)]
    pub fn code(&self) -> Option<u32> {
        self.code
    }

    /// Maps the base of this catcher using a new base string.
    #[napi]
    pub fn map_base(&mut self, new_base: String) {
        let clean = if !new_base.starts_with('/') {
            format!("/{new_base}")
        } else {
            new_base
        };
        self.base = clean;
    }
}

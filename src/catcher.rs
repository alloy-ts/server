//! NAPI bindings for Rocket's [`Catcher`](rocket::Catcher) struct.
//!
//! An error-catching route. Catchers run when errors (e.g. 404, 500) are
//! produced by the application.

use napi_derive::napi;

/// NAPI object representation of Rocket's `Catcher`.
#[napi(object)]
pub struct Catcher {
    pub name: Option<String>,
    pub base: String,
    pub code: Option<u32>,
}

/// Creates a new `Catcher` metadata object.
#[napi]
pub fn create_catcher(
    code: Option<u32>,
    name: Option<String>,
    base: Option<String>,
) -> Catcher {
    Catcher {
        name,
        base: base.unwrap_or_else(|| "/".to_string()),
        code,
    }
}

//! # NAPI bindings for Rocket [`Catcher`] (`rocket::Catcher`).

use napi_derive::napi;
use rocket::http::uri::Origin;

/// An error catching route binding for Rocket's `Catcher`.
///
/// Catchers run when errors are produced by the application (e.g., 404 Not Found,
/// 500 Internal Server Error, or default catchers).
#[napi]
#[derive(Clone, Debug)]
pub struct Catcher {
    /// The name of this catcher, if given.
    #[napi(readonly)]
    pub name: Option<String>,

    /// The HTTP status code to match against (e.g. 404, 500), or `None` for default.
    #[napi(readonly)]
    pub code: Option<u32>,

    /// The base mount point for this catcher.
    #[napi(readonly)]
    pub base: String,
}

#[napi]
impl Catcher {
    /// Create a new `Catcher` for a specific status code or default (when `code` is null/undefined).
    #[napi(constructor)]
    pub fn new(
        code: Option<u32>,
        name: Option<String>,
        base: Option<String>,
    ) -> napi::Result<Self> {
        if let Some(c) = code {
            if !(400..600).contains(&c) {
                return Err(napi::Error::new(
                    napi::Status::InvalidArg,
                    format!("Status code {c} must be in the HTTP error range [400, 600)"),
                ));
            }
        }

        let base_uri = base.unwrap_or_else(|| "/".to_string());
        let origin = Origin::parse_owned(base_uri.clone())
            .map_err(|e| napi::Error::new(napi::Status::InvalidArg, e.to_string()))?
            .into_normalized();

        Ok(Self {
            name,
            code,
            base: origin.path().to_string(),
        })
    }

    /// Create a catcher for a specific status code (e.g. 404 or 500).
    #[napi]
    pub fn for_code(code: u32, name: Option<String>) -> napi::Result<Self> {
        Self::new(Some(code), name, None)
    }

    /// Create a default catcher matching any unhandled error status.
    #[napi]
    pub fn default_catcher(name: Option<String>) -> Self {
        Self {
            name: name.or_else(|| Some("<Rocket Catcher>".to_string())),
            code: None,
            base: "/".to_string(),
        }
    }

    /// Maps the base URI of this catcher to a new base path.
    #[napi]
    pub fn map_base(&mut self, base: String) -> napi::Result<()> {
        let origin = Origin::parse_owned(base)
            .map_err(|e| napi::Error::new(napi::Status::InvalidArg, e.to_string()))?
            .into_normalized();
        self.base = origin.path().to_string();
        Ok(())
    }
}

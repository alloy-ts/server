use napi::bindgen_prelude::*;
use napi_derive::napi;

/// NAPI wrapper around Rocket's `Catcher` struct.
#[napi(js_name = "Catcher")]
#[derive(Clone, Debug)]
pub struct Catcher {
    /// The name of this catcher, if one was given.
    pub name: Option<String>,
    /// The mount point base path.
    pub base: String,
    /// The HTTP status code to match against (e.g. 404, 500), or null for default catchers.
    pub code: Option<u32>,
}

#[napi]
impl Catcher {
    /// Creates a Catcher for the given HTTP status code or null for default.
    #[napi(constructor)]
    pub fn new(code: Option<u32>, name: Option<String>) -> napi::Result<Self> {
        if let Some(c) = code {
            if !(400..600).contains(&c) {
                return Err(napi::Error::new(
                    Status::InvalidArg,
                    format!("Catcher code must be in range [400, 600), got: {c}"),
                ));
            }
        }

        Ok(Self {
            name,
            base: "/".to_string(),
            code,
        })
    }

    /// Creates a status-specific Catcher.
    #[napi(factory)]
    pub fn status(code: u32) -> napi::Result<Self> {
        Self::new(Some(code), None)
    }

    /// Creates a default catch-all Catcher.
    #[napi(factory)]
    pub fn default_catcher() -> napi::Result<Self> {
        Self::new(None, Some("default".to_string()))
    }
}

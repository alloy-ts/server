//! NAPI bindings for Rocket's [`RouteUri`] (`rocket::route::RouteUri`).
//!
//! A route URI is composed of:
//! - `base`: The mount point (e.g. `/` or `/base`).
//! - `unmounted_origin`: The URI pattern without the base mount point.
//! - `origin`: The full canonical URI with the base mount point.

use napi_derive::napi;

/// NAPI mirror of Rocket's `RouteUri`.
#[napi]
pub struct RouteUri {
    base: String,
    unmounted_origin: String,
    origin: String,
}

#[napi]
impl RouteUri {
    /// Creates a new `RouteUri` from base and uri origin path.
    #[napi(constructor)]
    pub fn new(base: String, uri: String) -> Self {
        let clean_base = if base.starts_with('/') {
            base
        } else {
            format!("/{base}")
        };
        let base_trim = clean_base.trim_end_matches('/');
        let base_str = if base_trim.is_empty() {
            "/"
        } else {
            base_trim
        };

        let clean_uri = if uri.starts_with('/') {
            uri
        } else {
            format!("/{uri}")
        };
        let canonical = if base_str == "/" {
            clean_uri.clone()
        } else {
            format!("{base_str}{clean_uri}")
        };

        Self {
            base: base_str.to_string(),
            unmounted_origin: clean_uri,
            origin: canonical,
        }
    }

    #[napi(getter)]
    pub fn base(&self) -> String {
        self.base.clone()
    }

    #[napi(getter)]
    pub fn unmounted_origin(&self) -> String {
        self.unmounted_origin.clone()
    }

    #[napi(getter)]
    pub fn origin(&self) -> String {
        self.origin.clone()
    }

    #[napi]
    pub fn path(&self) -> String {
        self.origin.split('?').next().unwrap_or("/").to_string()
    }

    #[napi]
    pub fn query(&self) -> Option<String> {
        let mut parts = self.origin.split('?');
        parts.next();
        parts.next().map(|q| q.to_string()).filter(|q| !q.is_empty())
    }

    #[napi]
    pub fn as_str(&self) -> String {
        self.origin.clone()
    }
}

//! NAPI bindings for Rocket's [`RouteUri`] (`rocket::route::RouteUri`).
//!
//! A route URI represents a request-matching URI composed of a static base mount point,
//! an unmounted origin path, and a canonical origin path.

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
    /// Creates a new `RouteUri` from base and unmounted origin strings.
    #[napi(constructor)]
    pub fn new(base: String, unmounted_origin: String) -> Self {
        let base_clean = if base.is_empty() {
            "/".to_string()
        } else if !base.starts_with('/') {
            format!("/{base}")
        } else {
            base
        };

        let unmounted_clean = if unmounted_origin.starts_with('/') {
            unmounted_origin
        } else {
            format!("/{unmounted_origin}")
        };

        let origin = if base_clean == "/" {
            unmounted_clean.clone()
        } else {
            let trimmed_base = base_clean.trim_end_matches('/');
            format!("{trimmed_base}{unmounted_clean}")
        };

        RouteUri {
            base: base_clean,
            unmounted_origin: unmounted_clean,
            origin,
        }
    }

    /// The path of the base mount point (e.g. `"/base"` or `"/"`).
    #[napi(getter)]
    pub fn base(&self) -> String {
        self.base.clone()
    }

    /// The URI without the base mount point (e.g. `"/foo/<bar>"`).
    #[napi(getter)]
    pub fn unmounted_origin(&self) -> String {
        self.unmounted_origin.clone()
    }

    /// The canonical URI with the base mount point (e.g. `"/base/foo/<bar>"`).
    #[napi(getter)]
    pub fn origin(&self) -> String {
        self.origin.clone()
    }

    /// The path part of this route URI.
    #[napi]
    pub fn path(&self) -> String {
        self.origin
            .split_once('?')
            .map_or(self.origin.as_str(), |(p, _)| p)
            .to_string()
    }

    /// The query part of this route URI, if present.
    #[napi]
    pub fn query(&self) -> Option<String> {
        self.origin.split_once('?').map(|(_, q)| q.to_string())
    }

    /// The full canonical URI string.
    #[napi]
    pub fn as_str(&self) -> String {
        self.origin.clone()
    }
}

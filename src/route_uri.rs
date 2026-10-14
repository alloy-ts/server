//! # NAPI bindings for Rocket [`RouteUri`] (`rocket::route::RouteUri`).

use napi_derive::napi;
use rocket::http::ext::IntoOwned;
use rocket::http::uri::Origin;

/// NAPI binding for Rocket's [`RouteUri`].
///
/// A route URI matches incoming requests and comprises `base` (mount point),
/// `unmountedOrigin` (route pattern without base), and `origin` (full canonical route URI).
#[napi]
#[derive(Clone, Debug)]
pub struct RouteUri {
    base_str: String,
    unmounted_str: String,
    origin_str: String,
}

#[napi]
impl RouteUri {
    /// Create a new `RouteUri` from base mount point and URI pattern.
    #[napi(constructor)]
    pub fn new(base: String, uri: String) -> napi::Result<Self> {
        let mut base_origin = Origin::parse(&base)
            .map_err(|e| napi::Error::new(napi::Status::InvalidArg, e.to_string()))?
            .into_normalized()
            .into_owned();
        base_origin.clear_query();

        let unmounted_origin = Origin::parse_route(&uri)
            .map_err(|e| napi::Error::new(napi::Status::InvalidArg, e.to_string()))?
            .into_normalized()
            .into_owned();

        let full = if base_origin.path() == "/" {
            unmounted_origin.to_string()
        } else {
            format!("{base_origin}{unmounted_origin}")
        };

        let origin = Origin::parse_route(&full)
            .map_err(|e| napi::Error::new(napi::Status::InvalidArg, e.to_string()))?
            .into_normalized()
            .into_owned();

        Ok(Self {
            base_str: base_origin.path().to_string(),
            unmounted_str: unmounted_origin.to_string(),
            origin_str: origin.to_string(),
        })
    }

    /// Static factory method to create a `RouteUri`.
    #[napi]
    pub fn create(base: String, uri: String) -> napi::Result<Self> {
        Self::new(base, uri)
    }

    /// The path of the base mount point (e.g. `"/api"` or `"/"`).
    #[napi(getter)]
    pub fn base(&self) -> String {
        self.base_str.clone()
    }

    /// The URI without the base mount point.
    #[napi(getter)]
    pub fn unmounted_origin(&self) -> String {
        self.unmounted_str.clone()
    }

    /// The full canonical route URI with base mount point.
    #[napi(getter)]
    pub fn origin(&self) -> String {
        self.origin_str.clone()
    }

    /// The path part of this route URI.
    #[napi]
    pub fn path(&self) -> String {
        Origin::parse(&self.origin_str)
            .map(|o| o.path().to_string())
            .unwrap_or_else(|_| self.origin_str.clone())
    }

    /// The query part of this route URI, if present.
    #[napi]
    pub fn query(&self) -> Option<String> {
        Origin::parse(&self.origin_str)
            .ok()
            .and_then(|o| o.query().map(|q| q.to_string()))
    }

    /// Returns the full URI string representation.
    #[napi]
    pub fn as_str(&self) -> String {
        self.origin_str.clone()
    }

    /// Returns true if the origin URI is normalized.
    #[napi]
    pub fn is_normalized(&self) -> bool {
        Origin::parse(&self.origin_str)
            .map(|o| o.is_normalized())
            .unwrap_or(false)
    }

    /// Calculates default rank of a route with this URI based on static/dynamic components.
    #[napi]
    pub fn default_rank(&self) -> i32 {
        let has_query = self.query().is_some();
        let path = self.path();
        let is_dynamic_path = path.contains('<');
        let is_dynamic_query = self.query().map_or(false, |q| q.contains('<'));

        match (is_dynamic_path, has_query, is_dynamic_query) {
            (false, false, _) => -9,
            (false, true, false) => -12,
            (false, true, true) => -10,
            (true, false, _) => -5,
            (true, true, false) => -8,
            (true, true, true) => -6,
        }
    }
}

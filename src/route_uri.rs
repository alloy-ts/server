//! NAPI bindings for Rocket's [`RouteUri`] (`rocket::route::RouteUri`).
//!
//! Exposes `RouteUri` as a NAPI class with constructor, property getters
//! (`base`, `path`, `query`, `as_str`, `unmounted_origin`), and methods (`is_normalized`, `to_string`).

use napi::bindgen_prelude::Error as NapiError;
use napi::Status;
use napi_derive::napi;

use rocket::http::Method;
use rocket::route::dummy_handler;
use rocket::route::RouteUri as RocketRouteUri;
use rocket::Route as RocketRoute;

/// A Rocket route URI matched against incoming requests.
#[napi]
pub struct RouteUri {
    pub base: String,
    pub path: String,
    pub query: Option<String>,
    pub as_str: String,
    pub unmounted_origin: String,
    pub(crate) inner: RocketRouteUri<'static>,
}

#[napi]
impl RouteUri {
    /// Create a new `RouteUri` from a base path and a route URI string.
    #[napi(constructor)]
    pub fn new(base: String, uri: String) -> napi::Result<Self> {
        let route = RocketRoute::ranked(None, Method::Get, &uri, dummy_handler);
        let route = route.map_base(|_| base).map_err(|e| {
            NapiError::new(Status::InvalidArg, format!("invalid route URI: {e}"))
        })?;
        Ok(Self::from_rocket(route.uri))
    }

    /// Convert a concrete `rocket::route::RouteUri` into a NAPI `RouteUri` wrapper.
    pub(crate) fn from_rocket(uri: RocketRouteUri<'static>) -> Self {
        let base = uri.base().to_string();
        let path = uri.path().to_string();
        let query = uri.query().map(|q| q.to_string());
        let as_str = uri.as_str().to_string();
        let unmounted_origin = uri.unmounted_origin.to_string();

        Self {
            base,
            path,
            query,
            as_str,
            unmounted_origin,
            inner: uri,
        }
    }

    /// Returns `true` if the underlying origin URI is normalized.
    #[napi]
    pub fn is_normalized(&self) -> bool {
        self.inner.origin.is_normalized()
    }

    /// Returns the full string representation of the route URI.
    #[napi]
    pub fn to_string(&self) -> String {
        self.inner.to_string()
    }

    #[napi(getter)]
    pub fn base(&self) -> String {
        self.base.clone()
    }

    #[napi(getter)]
    pub fn path(&self) -> String {
        self.path.clone()
    }

    #[napi(getter)]
    pub fn query(&self) -> Option<String> {
        self.query.clone()
    }

    #[napi(getter)]
    pub fn as_str(&self) -> String {
        self.as_str.clone()
    }

    #[napi(getter)]
    pub fn unmounted_origin(&self) -> String {
        self.unmounted_origin.clone()
    }
}

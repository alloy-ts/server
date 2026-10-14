// NAPI bindings for Rocket's RouteUri (rocket::route::RouteUri).

use napi_derive::napi;
use rocket::http::Method;
use rocket::route::{dummy_handler, Route as RocketRoute, RouteUri as RocketRouteUri};

/// A wrapper around Rocket's RouteUri.
#[napi]
pub struct RouteUri {
    pub(crate) inner: RocketRouteUri<'static>,
}

#[napi]
impl RouteUri {
    /// Create a new RouteUri given base and uri.
    #[napi(constructor)]
    pub fn new(base: String, uri: String) -> napi::Result<Self> {
        let route = RocketRoute::ranked(None, Method::Get, &uri, dummy_handler);
        let rebased = route
            .map_base(|_| base)
            .map_err(|e| napi::Error::new(napi::Status::GenericFailure, format!("invalid URI or base: {e}")))?;
        Ok(Self {
            inner: rebased.uri,
        })
    }

    /// The path of the base mount point of this route URI as a string.
    #[napi(getter)]
    pub fn base(&self) -> String {
        self.inner.base().to_string()
    }

    /// The path part of this route URI as a string.
    #[napi(getter)]
    pub fn path(&self) -> String {
        self.inner.path().to_string()
    }

    /// The query part of this route URI, if there is one.
    #[napi(getter)]
    pub fn query(&self) -> Option<String> {
        self.inner.query().map(|q| q.to_string())
    }

    /// The full URI as a string.
    #[napi(getter)]
    pub fn as_str(&self) -> String {
        self.inner.as_str().to_string()
    }

    /// The URI without the base mount point.
    #[napi(getter)]
    pub fn unmounted_origin(&self) -> String {
        self.inner.unmounted_origin.to_string()
    }

    /// The URI with the base mount point (canonical route URI).
    #[napi(getter)]
    pub fn origin(&self) -> String {
        self.inner.origin.to_string()
    }
}

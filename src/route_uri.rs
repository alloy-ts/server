//! NAPI bindings for Rocket's [`RouteUri`](rocket::route::RouteUri) type.

use napi_derive::napi;
use rocket::route::RouteUri as RocketRouteUri;

/// A NAPI wrapper around Rocket's [`RouteUri`](rocket::route::RouteUri).
#[napi]
#[derive(Clone)]
pub struct RouteUri {
    pub(crate) inner: RocketRouteUri<'static>,
}

#[napi]
impl RouteUri {
    /// Get the path of the base mount point of this route URI.
    #[napi(getter)]
    pub fn base(&self) -> String {
        self.inner.base().to_string()
    }

    /// Get the path part of this route URI.
    #[napi(getter)]
    pub fn path(&self) -> String {
        self.inner.path().to_string()
    }

    /// Get the query part of this route URI, if any.
    #[napi(getter)]
    pub fn query(&self) -> Option<String> {
        self.inner.query().map(|q| q.to_string())
    }

    /// Get the full URI string.
    #[napi(getter)]
    pub fn as_str(&self) -> String {
        self.inner.as_str().to_string()
    }
}

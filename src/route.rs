//! NAPI bindings for Rocket's [`Route`] (`rocket::Route`).
//!
//! Exposes `Route` as a NAPI class with constructors, property getters,
//! static factory methods (`new`, `ranked`), and methods (`map_base`, `to_string`).

use napi::bindgen_prelude::Error as NapiError;
use napi::Status;
use napi_derive::napi;

use rocket::http::Method;
use rocket::route::dummy_handler;
use rocket::Route as RocketRoute;

/// A request-handling Rocket route metadata object.
#[napi]
#[derive(Clone)]
pub struct Route {
    pub name: Option<String>,
    pub method: String,
    pub uri: String,
    pub rank: i64,
    pub format: Option<String>,
    pub(crate) inner: RocketRoute,
}

#[napi]
impl Route {
    /// Create a new route with the given method and URI, with a base of `/` and
    /// a computed default rank.
    #[napi(constructor)]
    pub fn new(method: String, uri: String) -> napi::Result<Self> {
        Self::ranked(None, method, uri)
    }

    /// Create a new route with the given optional rank, method, and URI with a base of `/`.
    /// If `rank` is `None`/`null`/`undefined`, the computed default rank is used.
    #[napi(factory)]
    pub fn ranked(rank: Option<i64>, method: String, uri: String) -> napi::Result<Self> {
        let m: Method = method.parse().map_err(|_| {
            NapiError::new(Status::InvalidArg, format!("invalid HTTP method: {method}"))
        })?;

        let r = rank.map(|r| r as isize);
        let route = RocketRoute::ranked(r, m, &uri, dummy_handler);

        Ok(Self::from_rocket(route))
    }

    /// Convert a concrete `rocket::Route` into a NAPI `Route` wrapper.
    pub(crate) fn from_rocket(route: RocketRoute) -> Self {
        let name = route.name.as_ref().map(|n| n.to_string());
        let method = route.method.as_str().to_string();
        let uri = route.uri.to_string();
        let rank = route.rank as i64;
        let format = route.format.as_ref().map(|f| f.to_string());

        Self {
            name,
            method,
            uri,
            rank,
            format,
            inner: route,
        }
    }

    /// Map the base URI of this route, returning a new `Route`.
    #[napi]
    pub fn map_base(&self, base: String) -> napi::Result<Self> {
        let route = self.inner.clone();
        let mapped = route.map_base(|_| base).map_err(|e| {
            NapiError::new(Status::InvalidArg, format!("invalid base URI: {e}"))
        })?;
        Ok(Self::from_rocket(mapped))
    }

    /// String representation of the route (e.g. `"GET /users/<id>"`).
    #[napi]
    pub fn to_string(&self) -> String {
        self.inner.to_string()
    }

    #[napi(getter)]
    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }

    #[napi(getter)]
    pub fn method(&self) -> String {
        self.method.clone()
    }

    #[napi(getter)]
    pub fn uri(&self) -> String {
        self.uri.clone()
    }

    #[napi(getter)]
    pub fn rank(&self) -> i64 {
        self.rank
    }

    #[napi(getter)]
    pub fn format(&self) -> Option<String> {
        self.format.clone()
    }
}

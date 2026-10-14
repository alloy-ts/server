//! NAPI bindings for Rocket's [`RouteUri`] (`rocket::route::RouteUri`).
//!
//! Exposes a parsed route URI's mount point (`base`), unmounted origin, canonical origin,
//! path, and query components.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rocket::http::Method;

/// NAPI wrapper around Rocket's [`RouteUri`].
#[napi]
#[derive(Clone)]
pub struct RouteUri {
    pub base: String,
    pub unmounted_origin: String,
    pub origin: String,
    pub path: String,
    pub query: Option<String>,
    pub as_str: String,
}

#[napi]
impl RouteUri {
    /// Constructs and parses a new `RouteUri` with an optional base mount point and URI.
    #[napi(constructor)]
    pub fn new(base: Option<String>, uri: String) -> napi::Result<Self> {
        let base_str = base.unwrap_or_else(|| "/".to_string());

        let route = rocket::Route::new(Method::Get, &uri, rocket::route::dummy_handler);
        let route = if base_str != "/" {
            route.map_base(|_| base_str.clone()).map_err(|e| {
                napi::Error::new(Status::InvalidArg, format!("invalid base URI: {e}"))
            })?
        } else {
            route
        };
        let route_uri = route.uri;

        Ok(RouteUri {
            base: route_uri.base().to_string(),
            unmounted_origin: route_uri.unmounted_origin.to_string(),
            origin: route_uri.origin.to_string(),
            path: route_uri.path().to_string(),
            query: route_uri.query().map(|q| q.to_string()),
            as_str: route_uri.as_str().to_string(),
        })
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
}

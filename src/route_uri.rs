//! NAPI bindings for Rocket's [`RouteUri`](rocket::route::RouteUri).
//!
//! Exposes route URI metadata (base, unmounted_origin, origin, path, query, as_str).

use napi::bindgen_prelude::Error as NapiError;
use napi::Status;
use napi_derive::napi;

use rocket::Route as RocketRoute;

/// Route URI metadata matching Rocket's `RouteUri`.
#[napi]
#[derive(Clone, Debug)]
pub struct RouteUri {
    pub base: String,
    pub unmounted_origin: String,
    pub origin: String,
    pub path: String,
    pub query: Option<String>,
}

#[napi]
impl RouteUri {
    /// Parse a route URI given a base mount point and unmounted route path.
    #[napi(constructor)]
    pub fn new(base: String, uri: String) -> napi::Result<Self> {
        let dummy = rocket::route::dummy_handler;
        let route = RocketRoute::new(rocket::http::Method::Get, &uri, dummy);
        let mapped = route.map_base(|_| base).map_err(|e| {
            NapiError::new(Status::InvalidArg, format!("invalid base URI: {e}"))
        })?;

        Ok(Self {
            base: mapped.uri.base().to_string(),
            unmounted_origin: mapped.uri.unmounted_origin.to_string(),
            origin: mapped.uri.origin.to_string(),
            path: mapped.uri.path().to_string(),
            query: mapped.uri.query().map(|q| q.to_string()),
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

    #[napi]
    pub fn as_str(&self) -> String {
        self.origin.clone()
    }
}

//! NAPI bindings for Rocket's [`RouteUri`] (`rocket::route::RouteUri`).
//!
//! Provides a JS-friendly mirror of Rocket's `RouteUri` struct, exposing
//! `base`, `unmountedOrigin`, `origin`, `path()`, `query()`, and `asStr()`.

use napi::bindgen_prelude::Error as NapiError;
use napi::Status;
use napi_derive::napi;

use rocket::http::Method;
use rocket::route::dummy_handler;
use rocket::Route as RocketRoute;

fn err<E: std::fmt::Display>(e: E) -> NapiError {
    NapiError::new(Status::GenericFailure, e.to_string())
}

/// NAPI mirror of Rocket's [`RouteUri`].
#[napi]
pub struct RouteUri {
    /// The path of the base mount point.
    pub base: String,
    /// The URI without the base mount point.
    pub unmounted_origin: String,
    /// The canonical route URI with the base mount point.
    pub origin: String,
}

impl RouteUri {
    fn build_rocket_route_uri(base: &str, uri: &str) -> napi::Result<rocket::route::RouteUri<'static>> {
        let route = RocketRoute::ranked(None, Method::Get, uri, dummy_handler);
        if base == "/" || base.is_empty() {
            Ok(route.uri)
        } else {
            let mapped = route.map_base(|_| base.to_string()).map_err(|e| err(e))?;
            Ok(mapped.uri)
        }
    }
}

#[napi]
impl RouteUri {
    /// Creates a new `RouteUri` from a base mount point and URI.
    #[napi(constructor)]
    pub fn new(base: String, uri: String) -> napi::Result<Self> {
        let route_uri = Self::build_rocket_route_uri(&base, &uri)?;
        Ok(RouteUri {
            base: route_uri.base().to_string(),
            unmounted_origin: route_uri.unmounted_origin.to_string(),
            origin: route_uri.origin.to_string(),
        })
    }

    /// The path part of this route URI.
    #[napi]
    pub fn path(&self) -> String {
        let route_uri = Self::build_rocket_route_uri(&self.base, &self.unmounted_origin);
        match route_uri {
            Ok(u) => u.path().to_string(),
            Err(_) => self.origin.clone(),
        }
    }

    /// The query part of this route URI, if present.
    #[napi]
    pub fn query(&self) -> Option<String> {
        let route_uri = Self::build_rocket_route_uri(&self.base, &self.unmounted_origin);
        match route_uri {
            Ok(u) => u.query().map(|q| q.to_string()),
            Err(_) => None,
        }
    }

    /// The full URI as a string.
    #[napi]
    pub fn as_str(&self) -> String {
        self.origin.clone()
    }
}

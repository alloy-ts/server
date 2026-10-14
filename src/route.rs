//! NAPI bindings for Rocket's [`Route`] (`rocket::Route`).
//!
//! This module exposes Rocket's `Route` data structure to TypeScript so callers
//! can inspect, construct, and rank routes dynamically or check for route
//! ranks and default ranking tables.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rocket::http::Method;
use rocket::route::dummy_handler;
use rocket::Route as RocketRoute;

/// NAPI wrapper around Rocket's [`Route`].
#[napi]
#[derive(Clone)]
pub struct Route {
    pub name: Option<String>,
    pub method: String,
    pub uri: String,
    pub rank: i64,
    pub format: Option<String>,
}

#[napi]
impl Route {
    /// Creates a new route with the given method, uri path, and calculated default rank.
    #[napi(constructor)]
    pub fn new(method: String, uri: String) -> napi::Result<Self> {
        Self::ranked(None, method, uri)
    }

    /// Creates a new route with an explicit rank, method, and URI path.
    /// If `rank` is `null`/`undefined`, Rocket calculates the default rank based on
    /// static/partial/wild path & query components.
    #[napi]
    pub fn ranked(rank: Option<i64>, method: String, uri: String) -> napi::Result<Self> {
        let m: Method = method.parse().map_err(|_| {
            napi::Error::new(Status::InvalidArg, format!("invalid HTTP method `{method}`"))
        })?;

        let route = RocketRoute::ranked(rank.map(|r| r as isize), m, &uri, dummy_handler);

        Ok(Route {
            name: route.name.map(|n| n.to_string()),
            method: route.method.as_str().to_string(),
            uri: route.uri.to_string(),
            rank: route.rank as i64,
            format: route.format.map(|f| f.to_string()),
        })
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

//! NAPI bindings for Rocket's [`Route`](rocket::Route).
//!
//! Exposes Rocket's `Route` structure, allowing Node.js/TypeScript callers to
//! construct, inspect, and query route metadata (name, method, uri, rank, format).

use napi::bindgen_prelude::Error as NapiError;
use napi::Status;
use napi_derive::napi;

use rocket::http::Method;
use rocket::Route as RocketRoute;

/// A request handling route metadata structure mirroring Rocket's `Route`.
#[napi]
#[derive(Clone, Debug)]
pub struct Route {
    pub name: Option<String>,
    pub method: String,
    pub uri: String,
    pub rank: i64,
    pub format: Option<String>,
}

#[napi]
impl Route {
    /// Create a new route with computed default rank.
    #[napi(constructor)]
    pub fn new(method: String, uri: String) -> napi::Result<Self> {
        Self::ranked(None, method, uri)
    }

    /// Create a route with a specific rank (or computed default rank if `rank` is null/undefined).
    #[napi]
    pub fn ranked(rank: Option<i64>, method: String, uri: String) -> napi::Result<Self> {
        let m: Method = method
            .parse()
            .map_err(|_| NapiError::new(Status::InvalidArg, format!("invalid method: {method}")))?;

        let dummy = rocket::route::dummy_handler;
        let r = match rank {
            Some(rk) => RocketRoute::ranked(rk as isize, m, &uri, dummy),
            None => RocketRoute::new(m, &uri, dummy),
        };

        Ok(Self {
            name: r.name.map(|n| n.to_string()),
            method: r.method.as_str().to_string(),
            uri: r.uri.to_string(),
            rank: r.rank as i64,
            format: r.format.map(|f| f.to_string()),
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

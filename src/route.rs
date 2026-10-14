//! NAPI bindings for Rocket's [`Route`] (`rocket::Route`).
//!
//! Exposes route metadata structures, default rank computation, and route
//! creation helpers mirroring `rocket::Route`.

use std::str::FromStr;
use napi_derive::napi;
use rocket::http::Method;
use rocket::route::dummy_handler;
use rocket::Route as RocketRoute;

/// NAPI mirror of Rocket's request handling `Route`.
#[napi]
pub struct Route {
    pub name: Option<String>,
    pub method: String,
    pub uri: String,
    pub rank: i64,
    pub format: Option<String>,
    pub base: String,
}

#[napi]
impl Route {
    /// Create a new route with method, URI, and optional rank / name / format.
    #[napi(constructor)]
    pub fn new(
        method: String,
        uri: String,
        name: Option<String>,
        rank: Option<i64>,
        format: Option<String>,
    ) -> Self {
        let m = Method::from_str(&method).unwrap_or(Method::Get);
        let inner = RocketRoute::ranked(
            rank.map(|r| r as isize),
            m,
            &uri,
            dummy_handler,
        );

        Route {
            name,
            method: inner.method.as_str().to_string(),
            uri: inner.uri.to_string(),
            rank: inner.rank as i64,
            format,
            base: inner.uri.base().to_string(),
        }
    }

    /// Creates a new route with a specific rank (or default rank if `None`).
    #[napi]
    pub fn ranked(rank: Option<i64>, method: String, uri: String) -> Self {
        Self::new(method, uri, None, rank, None)
    }

    /// Maps the base URI of this route.
    #[napi]
    pub fn map_base(&mut self, new_base: String) {
        self.base = new_base;
    }
}

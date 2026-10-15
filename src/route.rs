//! # NAPI bindings for Rocket [`Route`] (`rocket::Route`).
//!
//! A request-handling route metadata structure in Rocket.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rocket::http::Method;
use rocket::Route as RocketRoute;

/// NAPI binding for Rocket's [`Route`].
#[napi]
#[derive(Clone, Debug)]
pub struct Route {
    #[napi]
    pub name: Option<String>,
    #[napi]
    pub method: String,
    #[napi]
    pub uri: String,
    #[napi]
    pub rank: i32,
    #[napi]
    pub format: Option<String>,
}

#[napi]
impl Route {
    /// Create a new `Route` with given parameters.
    #[napi(constructor)]
    pub fn new(
        method: String,
        uri: String,
        name: Option<String>,
        rank: Option<i32>,
        format: Option<String>,
    ) -> napi::Result<Self> {
        let m: Method = method.parse().map_err(|_| {
            napi::Error::new(Status::GenericFailure, format!("invalid method: {method}"))
        })?;
        let dummy_handler = rocket::route::dummy_handler;
        let rocket_route = if let Some(r) = rank {
            RocketRoute::ranked(r as isize, m, &uri, dummy_handler)
        } else {
            RocketRoute::new(m, &uri, dummy_handler)
        };

        Ok(Self {
            name: name.or_else(|| rocket_route.name.map(|n| n.to_string())),
            method: rocket_route.method.as_str().to_string(),
            uri: rocket_route.uri.to_string(),
            rank: rocket_route.rank as i32,
            format: format.or_else(|| rocket_route.format.map(|f| f.to_string())),
        })
    }

    /// Creates a new route with default computed rank.
    #[napi]
    pub fn create(method: String, uri: String) -> napi::Result<Self> {
        Self::new(method, uri, None, None, None)
    }

    /// Creates a new route with explicit rank.
    #[napi]
    pub fn ranked(rank: Option<i32>, method: String, uri: String) -> napi::Result<Self> {
        Self::new(method, uri, None, rank, None)
    }

    /// Maps the base URI of this route.
    #[napi]
    pub fn map_base(&mut self, base: String) -> napi::Result<()> {
        let m: Method = self
            .method
            .parse()
            .map_err(|_| napi::Error::new(Status::GenericFailure, "invalid method"))?;
        let dummy_handler = rocket::route::dummy_handler;
        let rocket_route = RocketRoute::ranked(self.rank as isize, m, &self.uri, dummy_handler);
        let mapped = rocket_route
            .map_base(|_| base)
            .map_err(|e| napi::Error::new(Status::GenericFailure, e.to_string()))?;
        self.uri = mapped.uri.to_string();
        Ok(())
    }
}

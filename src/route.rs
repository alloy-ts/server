//! NAPI bindings for Rocket's [`Route`] (`rocket::Route`).
//!
//! Exposes route metadata structures, default rank computation, and route
//! creation helpers mirroring `rocket::Route`.

use std::str::FromStr;
use napi::bindgen_prelude::*;
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
    pub origin: String,
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
    ) -> Result<Self> {
        let m = Method::from_str(&method).unwrap_or(Method::Get);
        let catch_uri = uri.clone();
        let catch_rank = rank;

        let inner = std::panic::catch_unwind(move || {
            RocketRoute::ranked(
                catch_rank.map(|r| r as isize),
                m,
                &catch_uri,
                dummy_handler,
            )
        })
        .map_err(|_| Error::new(Status::InvalidArg, format!("invalid route URI: {uri}")))?;

        Ok(Route {
            name,
            method: inner.method.as_str().to_string(),
            uri: inner.uri.to_string(),
            rank: inner.rank as i64,
            format,
            base: inner.uri.base().to_string(),
            origin: uri,
        })
    }

    /// Creates a new route with a specific rank (or default rank if `None`).
    #[napi]
    pub fn ranked(rank: Option<i64>, method: String, uri: String) -> Result<Self> {
        Self::new(method, uri, None, rank, None)
    }

    /// Maps the base URI of this route.
    #[napi]
    pub fn map_base(&mut self, new_base: String) {
        let b = new_base.trim_end_matches('/').to_string();
        let o = if self.origin.starts_with('/') {
            self.origin.clone()
        } else {
            format!("/{}", self.origin)
        };
        self.uri = format!("{b}{o}");
        self.base = new_base;
    }
}

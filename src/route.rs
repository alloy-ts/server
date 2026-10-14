//! NAPI bindings for Rocket's [`Route`] (`rocket::Route`).
//!
//! This module exposes Rocket's `Route` structure and its constructors /
//! metadata (name, method, uri, rank, format) so Node.js callers can construct,
//! inspect, and manipulate routes programmatically.

use std::str::FromStr;

use napi::bindgen_prelude::Error as NapiError;
use napi::Status;
use napi_derive::napi;

use rocket::http::{MediaType, Method};
use rocket::Route as RocketRoute;

fn err<E: std::fmt::Display>(e: E) -> NapiError {
    NapiError::new(Status::GenericFailure, e.to_string())
}

/// NAPI wrapper around Rocket's [`Route`].
#[napi]
pub struct Route {
    name: Option<String>,
    method: String,
    uri: String,
    rank: isize,
    format: Option<String>,
}

#[napi]
impl Route {
    /// Creates a new route with the given method, path, and default rank.
    #[napi(constructor)]
    pub fn new(method: String, uri: String) -> napi::Result<Self> {
        Self::ranked(None, method, uri)
    }

    /// Creates a new route with an optional rank, method, and URI path.
    #[napi]
    pub fn ranked(rank: Option<i64>, method: String, uri: String) -> napi::Result<Self> {
        let m: Method = method
            .parse()
            .map_err(|_| NapiError::new(Status::InvalidArg, format!("invalid HTTP method `{method}`")))?;

        let r_opt: Option<isize> = rank.map(|r| r as isize);

        // Dummy handler for metadata inspection/construction
        let rocket_route = RocketRoute::ranked(r_opt, m, &uri, rocket::route::dummy_handler);

        Ok(Self {
            name: rocket_route.name.map(|n| n.to_string()),
            method: rocket_route.method.as_str().to_string(),
            uri: rocket_route.uri.to_string(),
            rank: rocket_route.rank,
            format: rocket_route.format.map(|f| f.to_string()),
        })
    }

    // -- getters / setters ------------------------------------------------

    #[napi(getter)]
    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }
    #[napi(setter)]
    pub fn set_name(&mut self, value: Option<String>) {
        self.name = value;
    }

    #[napi(getter)]
    pub fn method(&self) -> String {
        self.method.clone()
    }
    #[napi(setter)]
    pub fn set_method(&mut self, value: String) -> napi::Result<()> {
        value
            .parse::<Method>()
            .map_err(|_| NapiError::new(Status::InvalidArg, format!("invalid HTTP method `{value}`")))?;
        self.method = value.to_ascii_uppercase();
        Ok(())
    }

    #[napi(getter)]
    pub fn uri(&self) -> String {
        self.uri.clone()
    }
    #[napi(setter)]
    pub fn set_uri(&mut self, value: String) -> napi::Result<()> {
        let m: Method = self.method.parse().unwrap_or(Method::Get);
        let route = RocketRoute::ranked(Some(self.rank), m, &value, rocket::route::dummy_handler);
        self.uri = route.uri.to_string();
        self.rank = route.rank;
        Ok(())
    }

    #[napi(getter)]
    pub fn rank(&self) -> i64 {
        self.rank as i64
    }
    #[napi(setter)]
    pub fn set_rank(&mut self, value: i64) {
        self.rank = value as isize;
    }

    #[napi(getter)]
    pub fn format(&self) -> Option<String> {
        self.format.clone()
    }
    #[napi(setter)]
    pub fn set_format(&mut self, value: Option<String>) -> napi::Result<()> {
        if let Some(f) = &value {
            // Support shorthand format extension like "json" -> "application/json"
            if let Some(mt) = MediaType::from_extension(f) {
                self.format = Some(mt.to_string());
            } else {
                MediaType::from_str(f).map_err(err)?;
                self.format = Some(f.clone());
            }
        } else {
            self.format = None;
        }
        Ok(())
    }
}

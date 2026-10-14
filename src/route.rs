//! NAPI bindings for Rocket's [`Route`] (`rocket::Route`).
//!
//! Exposes Rocket's `Route` structure, allowing Node.js callers to inspect,
//! construct, rank, and configure route metadata (method, URI, rank, format,
//! name) for Rocket applications.

use napi::bindgen_prelude::Error as NapiError;
use napi::Status;
use napi_derive::napi;

use rocket::http::uri::Origin;
use rocket::http::{MediaType, Method};
use rocket::route::dummy_handler;
use rocket::Route as RocketRoute;

/// NAPI binding for Rocket's [`Route`].
#[napi]
pub struct Route {
    name: Option<String>,
    method: String,
    uri: String,
    rank: i64,
    format: Option<String>,
}

#[napi]
impl Route {
    /// Creates a new route with the given method and URI with a computed default rank.
    #[napi(constructor)]
    pub fn new(method: String, uri: String) -> napi::Result<Self> {
        let m: Method = method
            .parse()
            .map_err(|_| NapiError::new(Status::InvalidArg, format!("invalid HTTP method `{method}`")))?;

        let uri_normalized = if uri.starts_with('/') {
            uri.clone()
        } else {
            format!("/{uri}")
        };

        if Origin::parse_route(&uri_normalized).is_err() {
            return Err(NapiError::new(
                Status::InvalidArg,
                format!("invalid route URI `{uri}`"),
            ));
        }

        let route = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            RocketRoute::new(m, &uri_normalized, dummy_handler)
        }))
        .map_err(|_| NapiError::new(Status::InvalidArg, format!("invalid route URI `{uri}`")))?;

        Ok(Self {
            name: route.name.map(|n| n.to_string()),
            method: route.method.as_str().to_string(),
            uri: route.uri.to_string(),
            rank: route.rank as i64,
            format: route.format.map(|f| f.to_string()),
        })
    }

    /// Creates a new route with an explicit rank (or default rank if rank is null/undefined).
    #[napi]
    pub fn ranked(rank: Option<i64>, method: String, uri: String) -> napi::Result<Self> {
        let m: Method = method
            .parse()
            .map_err(|_| NapiError::new(Status::InvalidArg, format!("invalid HTTP method `{method}`")))?;

        let uri_normalized = if uri.starts_with('/') {
            uri.clone()
        } else {
            format!("/{uri}")
        };

        if Origin::parse_route(&uri_normalized).is_err() {
            return Err(NapiError::new(
                Status::InvalidArg,
                format!("invalid route URI `{uri}`"),
            ));
        }

        let route = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            RocketRoute::ranked(rank.map(|r| r as isize), m, &uri_normalized, dummy_handler)
        }))
        .map_err(|_| NapiError::new(Status::InvalidArg, format!("invalid route URI `{uri}`")))?;

        Ok(Self {
            name: route.name.map(|n| n.to_string()),
            method: route.method.as_str().to_string(),
            uri: route.uri.to_string(),
            rank: route.rank as i64,
            format: route.format.map(|f| f.to_string()),
        })
    }

    // -- getters & setters ------------------------------------------------

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

    #[napi(getter)]
    pub fn rank(&self) -> i64 {
        self.rank
    }
    #[napi(setter)]
    pub fn set_rank(&mut self, value: i64) {
        self.rank = value;
    }

    #[napi(getter)]
    pub fn format(&self) -> Option<String> {
        self.format.clone()
    }
    #[napi(setter)]
    pub fn set_format(&mut self, value: Option<String>) -> napi::Result<()> {
        if let Some(f) = &value {
            MediaType::parse_flexible(f)
                .ok_or_else(|| NapiError::new(Status::InvalidArg, format!("invalid media type `{f}`")))?;
        }
        self.format = value;
        Ok(())
    }
}

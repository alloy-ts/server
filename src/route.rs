//! # Rocket Route - NAPI Bindings (`src/route.rs`)
//!
//! Exposes Rocket's `Route` structure (`rocket::Route`) to TypeScript.

use napi::bindgen_prelude::Error as NapiError;
use napi::Status;
use napi_derive::napi;

use rocket::http::{MediaType, Method};
use rocket::route::dummy_handler;
use rocket::Route as RocketRoute;

/// NAPI binding for Rocket's `Route` structure.
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
    /// Create a new `Route` with computed default rank matching `Route::new(method, uri, handler)`.
    #[napi(constructor)]
    pub fn new(method: String, uri: String) -> napi::Result<Self> {
        Self::ranked(None, method, uri)
    }

    /// Creates a new route with the given method, uri, and default computed rank.
    #[napi]
    pub fn new_route(method: String, uri: String) -> napi::Result<Self> {
        Self::new(method, uri)
    }

    /// Creates a new route with the given rank (or computed default if `None`), method, and uri.
    #[napi]
    pub fn ranked(rank: Option<i64>, method: String, uri: String) -> napi::Result<Self> {
        let m: Method = method
            .parse()
            .map_err(|_| NapiError::new(Status::InvalidArg, format!("invalid HTTP method: {method}")))?;

        let rank_isize = rank.map(|r| r as isize);
        let uri_clone = uri.clone();
        let route_result = std::panic::catch_unwind(move || {
            RocketRoute::ranked(rank_isize, m, &uri_clone, dummy_handler)
        });

        let route = route_result.map_err(|_| {
            NapiError::new(Status::InvalidArg, format!("invalid route URI: {uri}"))
        })?;

        Ok(Self::from_rocket(&route))
    }

    /// Creates a `Route` with explicit name, rank, format, method, and URI.
    #[napi]
    pub fn create(
        name: Option<String>,
        method: String,
        uri: String,
        rank: Option<i64>,
        format: Option<String>,
    ) -> napi::Result<Self> {
        let mut route = Self::ranked(rank, method, uri)?;
        route.name = name;
        if let Some(ref f) = format {
            MediaType::parse_flexible(f)
                .ok_or_else(|| NapiError::new(Status::InvalidArg, format!("invalid media type: {f}")))?;
            route.format = format;
        }
        Ok(route)
    }

    // -- getters & setters --------------------------------------------------

    #[napi(getter)]
    pub fn get_name(&self) -> Option<String> {
        self.name.clone()
    }

    #[napi(setter)]
    pub fn set_name(&mut self, name: Option<String>) {
        self.name = name;
    }

    #[napi(getter)]
    pub fn get_method(&self) -> String {
        self.method.clone()
    }

    #[napi(setter)]
    pub fn set_method(&mut self, method: String) -> napi::Result<()> {
        method
            .parse::<Method>()
            .map_err(|_| NapiError::new(Status::InvalidArg, format!("invalid HTTP method: {method}")))?;
        self.method = method.to_ascii_uppercase();
        Ok(())
    }

    #[napi(getter)]
    pub fn get_uri(&self) -> String {
        self.uri.clone()
    }

    #[napi(getter)]
    pub fn get_rank(&self) -> i64 {
        self.rank
    }

    #[napi(setter)]
    pub fn set_rank(&mut self, rank: i64) {
        self.rank = rank;
    }

    #[napi(getter)]
    pub fn get_format(&self) -> Option<String> {
        self.format.clone()
    }

    #[napi(setter)]
    pub fn set_format(&mut self, format: Option<String>) -> napi::Result<()> {
        if let Some(ref f) = format {
            MediaType::parse_flexible(f)
                .ok_or_else(|| NapiError::new(Status::InvalidArg, format!("invalid media type: {f}")))?;
        }
        self.format = format;
        Ok(())
    }
}

impl Route {
    pub(crate) fn from_rocket(r: &RocketRoute) -> Self {
        Self {
            name: r.name.as_ref().map(|n| n.to_string()),
            method: r.method.as_str().to_string(),
            uri: r.uri.to_string(),
            rank: r.rank as i64,
            format: r.format.as_ref().map(|f| f.to_string()),
        }
    }
}

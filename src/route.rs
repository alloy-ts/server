//! NAPI bindings for Rocket's [`Route`] (`rocket::Route`).
//!
//! Provides a JS-friendly mirror of Rocket's `Route` struct, exposing
//! `name`, `method`, `uri`, `rank`, and `format` alongside `new()`, `ranked()`,
//! and `map_base()`.

use napi::bindgen_prelude::Error as NapiError;
use napi::Status;
use napi_derive::napi;

use rocket::http::Method;
use rocket::route::dummy_handler;
use rocket::Route as RocketRoute;

fn err<E: std::fmt::Display>(e: E) -> NapiError {
    NapiError::new(Status::GenericFailure, e.to_string())
}

/// NAPI mirror of Rocket's [`Route`].
#[napi]
pub struct Route {
    pub name: Option<String>,
    pub method: String,
    pub uri: String,
    pub rank: i64,
    pub format: Option<String>,
}

#[napi]
impl Route {
    /// Creates a new route with method, uri, and default computed rank.
    #[napi(constructor)]
    pub fn new(method: String, uri: String) -> napi::Result<Self> {
        Self::ranked(None, method, uri)
    }

    /// Creates a new route with optional rank, method, and uri.
    #[napi]
    pub fn ranked(rank: Option<i64>, method: String, uri: String) -> napi::Result<Self> {
        let m = method
            .parse::<Method>()
            .map_err(|_| NapiError::new(Status::InvalidArg, format!("invalid HTTP method `{method}`")))?;
        let r_rank = rank.map(|r| r as isize);
        let route = RocketRoute::ranked(r_rank, m, &uri, dummy_handler);

        Ok(Route {
            name: route.name.map(|n| n.to_string()),
            method: route.method.as_str().to_string(),
            uri: route.uri.to_string(),
            rank: route.rank as i64,
            format: route.format.map(|f| f.to_string()),
        })
    }

    /// Maps the base of this route using a prefix string.
    #[napi]
    pub fn map_base(&self, base: String) -> napi::Result<Self> {
        let m = self
            .method
            .parse::<Method>()
            .map_err(|_| NapiError::new(Status::InvalidArg, format!("invalid HTTP method `{}`", self.method)))?;
        let route = RocketRoute::ranked(Some(self.rank as isize), m, &self.uri, dummy_handler);
        let mapped = route
            .map_base(|current_base| format!("{}{}", base, current_base))
            .map_err(|e| err(e))?;

        Ok(Route {
            name: self.name.clone(),
            method: mapped.method.as_str().to_string(),
            uri: mapped.uri.to_string(),
            rank: mapped.rank as i64,
            format: self.format.clone(),
        })
    }

    #[napi(getter)]
    pub fn get_name(&self) -> Option<String> {
        self.name.clone()
    }
    #[napi(setter)]
    pub fn set_name(&mut self, value: Option<String>) {
        self.name = value;
    }

    #[napi(getter)]
    pub fn get_method(&self) -> String {
        self.method.clone()
    }
    #[napi(setter)]
    pub fn set_method(&mut self, value: String) -> napi::Result<()> {
        value
            .parse::<Method>()
            .map_err(|_| NapiError::new(Status::InvalidArg, format!("invalid HTTP method `{value}`")))?;
        self.method = value;
        Ok(())
    }

    #[napi(getter)]
    pub fn get_uri(&self) -> String {
        self.uri.clone()
    }
    #[napi(setter)]
    pub fn set_uri(&mut self, value: String) {
        self.uri = value;
    }

    #[napi(getter)]
    pub fn get_rank(&self) -> i64 {
        self.rank
    }
    #[napi(setter)]
    pub fn set_rank(&mut self, value: i64) {
        self.rank = value;
    }

    #[napi(getter)]
    pub fn get_format(&self) -> Option<String> {
        self.format.clone()
    }
    #[napi(setter)]
    pub fn set_format(&mut self, value: Option<String>) {
        self.format = value;
    }
}

//! NAPI bindings for Rocket's [`Route`] (`rocket::Route`).
//!
//! Exposes a `Route` class so JS callers can inspect and construct Rocket route
//! metadata (name, method, uri, rank, format).

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rocket::http::Method;

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
    #[napi(constructor)]
    pub fn new(method: String, uri: String, name: Option<String>, rank: Option<i64>, format: Option<String>) -> napi::Result<Self> {
        let m: Method = method
            .parse()
            .map_err(|_| Error::new(Status::InvalidArg, format!("invalid HTTP method: {method}")))?;

        let r = rocket::Route::ranked(
            rank.map(|r| r as isize),
            m,
            &uri,
            rocket::route::dummy_handler,
        );

        Ok(Route {
            name,
            method: r.method.as_str().to_string(),
            uri: r.uri.to_string(),
            rank: r.rank as i64,
            format,
        })
    }

    #[napi]
    pub fn ranked(rank: Option<i64>, method: String, uri: String) -> napi::Result<Self> {
        Self::new(method, uri, None, rank, None)
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

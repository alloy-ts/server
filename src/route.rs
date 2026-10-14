//! NAPI bindings for Rocket's [`Route`] (`rocket::route::Route`).
//!
//! This module provides JavaScript / TypeScript bindings for Rocket's
//! request-handling route representation, allowing callers to inspect,
//! construct, rank, and manipulate routes programmatically.

use napi::bindgen_prelude::*;
use napi_derive::napi;

use rocket::http::{MediaType, Method};
use rocket::route::{Handler, Outcome as RouteOutcome};
use rocket::{Data, Request};

/// Dummy handler used when constructing NAPI Route instances.
#[derive(Clone)]
struct DummyRouteHandler;

#[rocket::async_trait]
impl Handler for DummyRouteHandler {
    async fn handle<'r>(&self, _req: &'r Request<'_>, _data: Data<'r>) -> RouteOutcome<'r> {
        RouteOutcome::Forward((_data, rocket::http::Status::NotFound))
    }
}

/// NAPI binding representing a Rocket request-handling route (`rocket::Route`).
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
    /// Creates a new route with the given method, path, and default rank.
    #[napi(constructor)]
    pub fn new(method: String, uri: String, name: Option<String>, format: Option<String>) -> napi::Result<Self> {
        let m: Method = method
            .parse()
            .map_err(|_| napi::Error::new(Status::GenericFailure, format!("invalid HTTP method: {method}")))?;

        let mut rocket_route = rocket::Route::new(m, &uri, DummyRouteHandler);
        if let Some(ref n) = name {
            rocket_route.name = Some(n.clone().into());
        }
        if let Some(ref f) = format {
            if let Ok(mt) = f.parse::<MediaType>() {
                rocket_route.format = Some(mt);
            }
        }

        Ok(Self {
            name,
            method: rocket_route.method.as_str().to_string(),
            uri: rocket_route.uri.to_string(),
            rank: rocket_route.rank as i64,
            format,
        })
    }

    /// Creates a new route with an explicit rank.
    #[napi]
    pub fn ranked(
        rank: Option<i64>,
        method: String,
        uri: String,
        name: Option<String>,
        format: Option<String>,
    ) -> napi::Result<Self> {
        let m: Method = method
            .parse()
            .map_err(|_| napi::Error::new(Status::GenericFailure, format!("invalid HTTP method: {method}")))?;

        let isize_rank = rank.map(|r| r as isize);
        let mut rocket_route = rocket::Route::ranked(isize_rank, m, &uri, DummyRouteHandler);
        if let Some(ref n) = name {
            rocket_route.name = Some(n.clone().into());
        }
        if let Some(ref f) = format {
            if let Ok(mt) = f.parse::<MediaType>() {
                rocket_route.format = Some(mt);
            }
        }

        Ok(Self {
            name,
            method: rocket_route.method.as_str().to_string(),
            uri: rocket_route.uri.to_string(),
            rank: rocket_route.rank as i64,
            format,
        })
    }

    /// Maps the base URI of this route using a prefix string.
    #[napi]
    pub fn map_base(&mut self, base: String) -> napi::Result<()> {
        let m: Method = self.method
            .parse()
            .map_err(|_| napi::Error::new(Status::GenericFailure, format!("invalid HTTP method: {}", self.method)))?;

        let rocket_route = rocket::Route::ranked(Some(self.rank as isize), m, &self.uri, DummyRouteHandler);
        let mapped = rocket_route
            .map_base(|b| format!("{}{}", base, b))
            .map_err(|e| napi::Error::new(Status::GenericFailure, e.to_string()))?;

        self.uri = mapped.uri.to_string();
        Ok(())
    }

    // Getters and Setters

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
            .map_err(|_| napi::Error::new(Status::GenericFailure, format!("invalid HTTP method: {value}")))?;
        self.method = value;
        Ok(())
    }

    #[napi(getter)]
    pub fn uri(&self) -> String {
        self.uri.clone()
    }

    #[napi(setter)]
    pub fn set_uri(&mut self, value: String) {
        self.uri = value;
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
    pub fn set_format(&mut self, value: Option<String>) {
        self.format = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_default_rank_computation() {
        let route = Route::new("GET".to_string(), "/".to_string(), Some("index".to_string()), None).unwrap();
        assert_eq!(route.method(), "GET");
        assert_eq!(route.uri(), "/");
        assert_eq!(route.rank(), -9);
        assert_eq!(route.name(), Some("index".to_string()));
    }

    #[test]
    fn test_route_ranked_construction() {
        let route = Route::ranked(
            Some(2),
            "POST".to_string(),
            "/route/<path..>?query".to_string(),
            Some("route_name".to_string()),
            Some("json".to_string()),
        )
        .unwrap();

        assert_eq!(route.name(), Some("route_name".to_string()));
        assert_eq!(route.method(), "POST");
        assert_eq!(route.rank(), 2);
        assert_eq!(route.format(), Some("json".to_string()));
    }

    #[test]
    fn test_route_map_base() {
        let mut route = Route::new("GET".to_string(), "/foo/bar".to_string(), None, None).unwrap();
        assert_eq!(route.uri(), "/foo/bar");
        route.map_base("/boo".to_string()).unwrap();
        assert_eq!(route.uri(), "/boo/foo/bar");
    }
}

//! NAPI bindings for Rocket's [`Route`] (`rocket::Route`).

use std::borrow::Cow;

use napi::bindgen_prelude::Error as NapiError;
use napi::Status;
use napi_derive::napi;

use rocket::http::{MediaType, Method};
use rocket::route::dummy_handler;
use rocket::Route as RocketRoute;

fn parse_method(method: &str) -> napi::Result<Method> {
    method
        .parse()
        .map_err(|_| NapiError::new(Status::InvalidArg, format!("invalid HTTP method: {method}")))
}

/// NAPI class mirroring `rocket::Route`.
#[napi]
pub struct Route {
    pub(crate) inner: RocketRoute,
}

#[napi]
impl Route {
    /// Create a new route with method and URI pattern with computed default rank.
    #[napi(constructor)]
    pub fn new(method: String, uri: String) -> napi::Result<Self> {
        let m = parse_method(&method)?;
        let route = RocketRoute::new(m, &uri, dummy_handler);
        Ok(Self { inner: route })
    }

    /// Create a new route with explicit rank (or default rank if null/undefined).
    #[napi]
    pub fn ranked(rank: Option<i64>, method: String, uri: String) -> napi::Result<Self> {
        let m = parse_method(&method)?;
        let route = RocketRoute::ranked(rank.map(|r| r as isize), m, &uri, dummy_handler);
        Ok(Self { inner: route })
    }

    /// The name of this route, if one was given.
    #[napi(getter)]
    pub fn name(&self) -> Option<String> {
        self.inner.name.as_ref().map(|n| n.to_string())
    }

    #[napi(setter)]
    pub fn set_name(&mut self, name: Option<String>) {
        self.inner.name = name.map(Cow::Owned);
    }

    /// The HTTP method this route matches against.
    #[napi(getter)]
    pub fn method(&self) -> String {
        self.inner.method.as_str().to_string()
    }

    /// The route URI pattern string.
    #[napi(getter)]
    pub fn uri(&self) -> String {
        self.inner.uri.to_string()
    }

    /// The rank of this route. Lower ranks have higher priorities.
    #[napi(getter)]
    pub fn rank(&self) -> i64 {
        self.inner.rank as i64
    }

    #[napi(setter)]
    pub fn set_rank(&mut self, rank: i64) {
        self.inner.rank = rank as isize;
    }

    /// The media type format string this route matches against, if any.
    #[napi(getter)]
    pub fn format(&self) -> Option<String> {
        self.inner.format.as_ref().map(|f| f.to_string())
    }

    #[napi(setter)]
    pub fn set_format(&mut self, format: Option<String>) -> napi::Result<()> {
        if let Some(f) = format {
            let mt = MediaType::parse_flexible(&f).ok_or_else(|| {
                NapiError::new(Status::InvalidArg, format!("invalid media type: {f}"))
            })?;
            self.inner.format = Some(mt);
        } else {
            self.inner.format = None;
        }
        Ok(())
    }

    /// String representation of the route.
    #[napi]
    pub fn to_string(&self) -> String {
        self.inner.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_new_default_ranks() {
        let route1 = Route::new("GET".to_string(), "/".to_string()).unwrap();
        assert_eq!(route1.rank(), -9);
        assert_eq!(route1.method(), "GET");
        assert_eq!(route1.uri(), "/");

        let route2 = Route::new("GET".to_string(), "/?foo".to_string()).unwrap();
        assert_eq!(route2.rank(), -12);

        let route3 = Route::new("GET".to_string(), "/a/<b>".to_string()).unwrap();
        assert_eq!(route3.rank(), -5);
    }

    #[test]
    fn test_route_ranked() {
        let route = Route::ranked(Some(1), "POST".to_string(), "/foo?bar".to_string()).unwrap();
        assert_eq!(route.rank(), 1);
        assert_eq!(route.method(), "POST");
        assert_eq!(route.uri(), "/foo?bar");

        let default_ranked = Route::ranked(None, "POST".to_string(), "/foo?bar".to_string()).unwrap();
        assert_eq!(default_ranked.rank(), -12);
    }

    #[test]
    fn test_route_getters_and_setters() {
        let mut route = Route::new("GET".to_string(), "/test".to_string()).unwrap();
        assert_eq!(route.name(), None);

        route.set_name(Some("test_route".to_string()));
        assert_eq!(route.name(), Some("test_route".to_string()));

        route.set_rank(5);
        assert_eq!(route.rank(), 5);

        assert_eq!(route.format(), None);
        assert!(route.set_format(Some("json".to_string())).is_ok());
        assert_eq!(route.format(), Some("application/json".to_string()));
    }
}

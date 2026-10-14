//! NAPI bindings for Rocket's [`RouteUri`](rocket::route::RouteUri).

use napi::bindgen_prelude::Error as NapiError;
use napi::Status;
use napi_derive::napi;

use rocket::http::Method;
use rocket::route::dummy_handler;
use rocket::route::RouteUri as RocketRouteUri;
use rocket::Route as RocketRoute;

/// NAPI class mirroring `rocket::route::RouteUri`.
#[napi]
pub struct RouteUri {
    pub(crate) inner: RocketRouteUri<'static>,
}

#[napi]
impl RouteUri {
    /// Parse a new `RouteUri` from a base mount point and route URI pattern.
    #[napi(constructor)]
    pub fn new(base: String, uri: String) -> napi::Result<Self> {
        let base_clone = base.clone();
        let uri_clone = uri.clone();
        let res = std::panic::catch_unwind(move || {
            let route = RocketRoute::new(Method::Get, &uri_clone, dummy_handler);
            if base_clone != "/" {
                let mapped = route.map_base(|_| base_clone).ok()?;
                Some(mapped.uri)
            } else {
                Some(route.uri)
            }
        });

        match res {
            Ok(Some(route_uri)) => Ok(Self { inner: route_uri }),
            _ => Err(NapiError::new(
                Status::InvalidArg,
                format!("invalid RouteUri base `{base}` or uri `{uri}`"),
            )),
        }
    }

    /// The base mount point path (`/`).
    #[napi(getter)]
    pub fn base(&self) -> String {
        self.inner.base().to_string()
    }

    /// The path part of this route URI.
    #[napi(getter)]
    pub fn path(&self) -> String {
        self.inner.path().to_string()
    }

    /// The query part of this route URI, if present.
    #[napi(getter)]
    pub fn query(&self) -> Option<String> {
        self.inner.query().map(|q| q.to_string())
    }

    /// The full URI as a string.
    #[napi]
    pub fn as_str(&self) -> String {
        self.inner.as_str().to_string()
    }

    /// Computed default rank for this route URI.
    #[napi(getter)]
    pub fn default_rank(&self) -> i64 {
        let dummy = RocketRoute::new(Method::Get, self.inner.as_str(), dummy_handler);
        dummy.rank as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_uri_parsed_values() {
        let uri = RouteUri::new("/base".to_string(), "/foo/bar?a=1".to_string()).unwrap();
        assert_eq!(uri.base(), "/base");
        assert_eq!(uri.path(), "/base/foo/bar");
        assert_eq!(uri.query(), Some("a=1".to_string()));
        assert_eq!(uri.as_str(), "/base/foo/bar?a=1");
    }

    #[test]
    fn test_route_uri_default_ranks() {
        let uri1 = RouteUri::new("/".to_string(), "/?foo".to_string()).unwrap();
        assert_eq!(uri1.default_rank(), -12);

        let uri2 = RouteUri::new("/".to_string(), "/".to_string()).unwrap();
        assert_eq!(uri2.default_rank(), -9);
    }
}

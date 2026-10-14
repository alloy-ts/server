//! NAPI bindings for Rocket's [`RouteUri`] (`rocket::route::RouteUri`).
//!
//! This module provides JavaScript / TypeScript bindings for Rocket's
//! route URIs, allowing callers to inspect base mount points, unmounted
//! origins, full paths, and query strings programmatically.

use napi::bindgen_prelude::*;
use napi_derive::napi;

use rocket::http::ext::IntoOwned;
use rocket::http::uri::Origin;

/// NAPI binding representing a Rocket route URI (`rocket::route::RouteUri`).
#[napi]
pub struct RouteUri {
    base: String,
    unmounted_origin: String,
    origin: String,
    query: Option<String>,
}

#[napi]
impl RouteUri {
    /// Creates a new RouteUri given a base path and route URI string.
    #[napi(constructor)]
    pub fn new(base: String, uri: String) -> napi::Result<Self> {
        let mut base_origin = Origin::parse(&base)
            .map_err(|e| napi::Error::new(Status::GenericFailure, e.to_string()))?
            .into_normalized()
            .into_owned();
        base_origin.clear_query();

        let unmounted = Origin::parse_route(&uri)
            .map_err(|e| napi::Error::new(Status::GenericFailure, e.to_string()))?
            .into_normalized()
            .into_owned();

        let full_origin = Origin::parse_route(&format!("{}/{}", base_origin, unmounted))
            .map_err(|e| napi::Error::new(Status::GenericFailure, e.to_string()))?
            .into_normalized()
            .into_owned();

        let query = full_origin.query().map(|q| q.as_str().to_string());

        Ok(Self {
            base: base_origin.path().as_str().to_string(),
            unmounted_origin: unmounted.to_string(),
            origin: full_origin.to_string(),
            query,
        })
    }

    /// The path of the base mount point of this route URI as a string.
    #[napi(getter)]
    pub fn base(&self) -> String {
        self.base.clone()
    }

    /// The path part of this route URI as a string.
    #[napi(getter)]
    pub fn path(&self) -> String {
        if let Some((p, _)) = self.origin.split_once('?') {
            p.to_string()
        } else {
            self.origin.clone()
        }
    }

    /// The query part of this route URI, if there is one.
    #[napi(getter)]
    pub fn query(&self) -> Option<String> {
        self.query.clone()
    }

    /// The URI without the base mount point.
    #[napi(getter)]
    pub fn unmounted_origin(&self) -> String {
        self.unmounted_origin.clone()
    }

    /// The full URI as a string.
    #[napi]
    pub fn as_str(&self) -> String {
        self.origin.clone()
    }

    /// Returns true if the origin URI is normalized.
    #[napi]
    pub fn is_normalized(&self) -> bool {
        Origin::parse(&self.origin)
            .map(|o| o.is_normalized())
            .unwrap_or(false)
    }

    /// Maps the base URI of this route URI using a prefix string.
    #[napi]
    pub fn map_base(&mut self, new_base_prefix: String) -> napi::Result<()> {
        let combined_base = format!("{}{}", new_base_prefix, self.base);
        let updated = RouteUri::new(combined_base, self.unmounted_origin.clone())?;
        self.base = updated.base;
        self.origin = updated.origin;
        self.query = updated.query;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_uri_parsing() {
        let route_uri = RouteUri::new("/".to_string(), "/foo/bar?a=1".to_string()).unwrap();
        assert_eq!(route_uri.base(), "/");
        assert_eq!(route_uri.path(), "/foo/bar");
        assert_eq!(route_uri.query(), Some("a=1".to_string()));
        assert_eq!(route_uri.as_str(), "/foo/bar?a=1");
    }

    #[test]
    fn test_route_uri_map_base() {
        let mut route_uri = RouteUri::new("/".to_string(), "/foo/bar?a=1".to_string()).unwrap();
        assert_eq!(route_uri.base(), "/");
        route_uri.map_base("/boo".to_string()).unwrap();
        assert_eq!(route_uri.base(), "/boo");
        assert_eq!(route_uri.path(), "/boo/foo/bar");
        assert_eq!(route_uri.as_str(), "/boo/foo/bar?a=1");
    }
}

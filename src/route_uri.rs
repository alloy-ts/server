//! NAPI bindings for Rocket's [`RouteUri`](rocket::route::RouteUri).
//!
//! Exposes `RouteUri` to inspect base mount points, unmounted origins, full
//! canonical route origins, path components, and query strings.

use napi::bindgen_prelude::Error as NapiError;
use napi::Status;
use napi_derive::napi;

use rocket::http::uri::Origin;
use rocket::http::Method;
use rocket::route::dummy_handler;
use rocket::Route as RocketRoute;

fn err<E: std::fmt::Display>(e: E) -> NapiError {
    NapiError::new(Status::InvalidArg, format!("invalid route URI: {e}"))
}

/// NAPI binding for Rocket's [`RouteUri`].
#[napi]
pub struct RouteUri {
    base: String,
    unmounted_origin: String,
    origin: String,
    path: String,
    query: Option<String>,
    as_str: String,
}

#[napi]
impl RouteUri {
    /// Creates a new `RouteUri` given a `base` mount point and a `uri` string.
    #[napi(constructor)]
    pub fn new(base: String, uri: String) -> napi::Result<Self> {
        let base_normalized = if base.starts_with('/') { base } else { format!("/{base}") };
        let uri_normalized = if uri.starts_with('/') { uri } else { format!("/{uri}") };

        if base_normalized.contains('?') || base_normalized.contains('#') {
            return Err(NapiError::new(
                Status::InvalidArg,
                "base mount point cannot contain a query or fragment",
            ));
        }

        // Validate base and URI before passing to Rocket
        let _parsed_base = Origin::parse(&base_normalized)
            .map_err(|e| NapiError::new(Status::InvalidArg, format!("invalid base URI: {e}")))?;
        let _parsed_uri = Origin::parse_route(&uri_normalized)
            .map_err(|e| NapiError::new(Status::InvalidArg, format!("invalid route URI: {e}")))?;

        let route = RocketRoute::new(Method::Get, &uri_normalized, dummy_handler);
        let route = if base_normalized != "/" {
            route.map_base(|_| base_normalized.clone()).map_err(err)?
        } else {
            route
        };
        let route_uri = route.uri;

        Ok(Self {
            base: route_uri.base().to_string(),
            unmounted_origin: route_uri.unmounted_origin.to_string(),
            origin: route_uri.origin.to_string(),
            path: route_uri.path().to_string(),
            query: route_uri.query().map(|q| q.to_string()),
            as_str: route_uri.as_str().to_string(),
        })
    }

    /// The path of the base mount point of this route URI.
    #[napi(getter)]
    pub fn base(&self) -> String {
        self.base.clone()
    }

    /// The URI without the base mount point.
    #[napi(getter)]
    pub fn unmounted_origin(&self) -> String {
        self.unmounted_origin.clone()
    }

    /// The canonical URI with the base mount point.
    #[napi(getter)]
    pub fn origin(&self) -> String {
        self.origin.clone()
    }

    /// The path part of this route URI.
    #[napi(getter)]
    pub fn path(&self) -> String {
        self.path.clone()
    }

    /// The query part of this route URI, if any.
    #[napi(getter)]
    pub fn query(&self) -> Option<String> {
        self.query.clone()
    }

    /// The full URI as a string.
    #[napi]
    pub fn as_str(&self) -> String {
        self.as_str.clone()
    }
}

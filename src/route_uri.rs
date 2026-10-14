//! Rocket RouteUri NAPI Bindings (`src/route_uri.rs`)
//!
//! Exposes Rocket's `RouteUri` structure to TypeScript, providing access to
//! route mount point (`base`), `unmounted_origin`, canonical `origin`, and helper
//! methods (`path()`, `query()`, `as_str()`, `is_normalized()`).

use napi::bindgen_prelude::Error as NapiError;
use napi::Status;
use napi_derive::napi;
use rocket::http::uri::Origin;
use rocket::http::Method;
use rocket::route::dummy_handler;
use rocket::Route as RocketRoute;

/// NAPI binding for Rocket's `RouteUri` structure.
#[napi]
#[derive(Clone)]
pub struct RouteUri {
    pub base: String,
    pub unmounted_origin: String,
    pub origin: String,
}

#[napi]
impl RouteUri {
    /// Parses a `RouteUri` given a base mount point (e.g. `"/base"`) and route pattern (e.g. `"/foo/<bar>?a=1"`).
    #[napi(constructor)]
    pub fn try_new(base: String, uri: String) -> napi::Result<Self> {
        let clean_base = if base.is_empty() {
            "/".to_string()
        } else if !base.starts_with('/') {
            format!("/{base}")
        } else {
            base
        };

        let clean_uri = if !uri.starts_with('/') {
            format!("/{uri}")
        } else {
            uri
        };

        // Validate base and uri as Origins
        let base_origin = Origin::parse(&clean_base).map_err(|e| {
            NapiError::new(Status::InvalidArg, format!("invalid base URI: {e}"))
        })?;

        let full_path = format!("{}{}", clean_base.trim_end_matches('/'), clean_uri);

        // Construct RocketRoute to derive Rocket's exact RouteUri canonicalization
        let route = RocketRoute::ranked(None, Method::Get, &full_path, dummy_handler);

        Ok(Self {
            base: base_origin.path().as_str().to_string(),
            unmounted_origin: clean_uri,
            origin: route.uri.to_string(),
        })
    }

    /// The base mount point of this route URI as a string.
    #[napi(getter)]
    pub fn get_base(&self) -> String {
        self.base.clone()
    }

    /// The unmounted origin (without base mount point) as a string.
    #[napi(getter)]
    pub fn get_unmounted_origin(&self) -> String {
        self.unmounted_origin.clone()
    }

    /// The canonical origin (with base mount point) as a string.
    #[napi(getter)]
    pub fn get_origin(&self) -> String {
        self.origin.clone()
    }

    /// The path part of this route URI.
    #[napi]
    pub fn path(&self) -> String {
        Origin::parse_route(&self.origin)
            .map(|o| o.path().as_str().to_string())
            .unwrap_or_else(|_| self.origin.clone())
    }

    /// The query part of this route URI, if present.
    #[napi]
    pub fn query(&self) -> Option<String> {
        Origin::parse_route(&self.origin)
            .ok()
            .and_then(|o| o.query().map(|q| q.as_str().to_string()))
    }

    /// Returns the full URI string representation.
    #[napi]
    pub fn as_str(&self) -> String {
        self.origin.clone()
    }

    /// Returns whether this URI is normalized according to Rocket's URI specification.
    #[napi]
    pub fn is_normalized(&self) -> bool {
        Origin::parse_route(&self.origin)
            .map(|o| o.is_normalized())
            .unwrap_or(false)
    }
}

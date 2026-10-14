//! # Rocket RouteUri NAPI Bindings
//!
//! NAPI mirror of Rocket's `RouteUri` struct representing base mount point,
//! unmounted origin, canonical origin, path, query, and default ranking.

use napi_derive::napi;
use rocket::http::Method;
use rocket::route::dummy_handler;
use rocket::Route as RocketRoute;

#[napi]
#[derive(Clone)]
pub struct RouteUri {
    pub base: String,
    pub unmounted_origin: String,
    pub origin: String,
}

#[napi]
impl RouteUri {
    /// Create a new `RouteUri` with the given base mount point and URI pattern.
    #[napi(constructor)]
    pub fn new(base: Option<String>, uri: String) -> napi::Result<Self> {
        let base_str = base.unwrap_or_else(|| "/".to_string());
        let route = RocketRoute::new(Method::Get, &uri, dummy_handler);
        let mapped = if base_str != "/" {
            route.map_base(|b| format!("{}{}", base_str, b))
                .map_err(|_| napi::Error::new(napi::Status::InvalidArg, "invalid base path"))?
        } else {
            route
        };

        let base_val = mapped.uri.base().to_string();
        let path_val = mapped.uri.path().to_string();
        let query_val = mapped.uri.query().map(|q| q.to_string());
        let origin_val = match &query_val {
            Some(q) => format!("{path_val}?{q}"),
            None => path_val.clone(),
        };

        let unmounted_val = if path_val.starts_with(&base_val) && base_val != "/" {
            let rest = &path_val[base_val.len()..];
            let unmounted_path = if rest.starts_with('/') { rest } else { "/" };
            match &query_val {
                Some(q) => format!("{unmounted_path}?{q}"),
                None => unmounted_path.to_string(),
            }
        } else {
            origin_val.clone()
        };

        Ok(Self {
            base: base_val,
            unmounted_origin: unmounted_val,
            origin: origin_val,
        })
    }

    /// Returns the base mount point path.
    #[napi(getter)]
    pub fn get_base(&self) -> String {
        self.base.clone()
    }

    /// Returns the path component of the canonical origin URI.
    #[napi]
    pub fn path(&self) -> String {
        let path = match self.origin.split_once('?') {
            Some((p, _)) => p,
            None => self.origin.as_str(),
        };
        path.to_string()
    }

    /// Returns the query string of the canonical origin URI, if present.
    #[napi]
    pub fn query(&self) -> Option<String> {
        self.origin.split_once('?').map(|(_, q)| q.to_string())
    }

    /// Returns the full canonical URI string representation.
    #[napi(js_name = "asStr")]
    pub fn as_str(&self) -> String {
        self.origin.clone()
    }

    /// Calculates the default route rank based on Rocket's ranking table.
    #[napi(js_name = "defaultRank")]
    pub fn default_rank(&self) -> i32 {
        crate::route::compute_default_rank(&self.origin)
    }
}

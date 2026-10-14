//! NAPI bindings for Rocket's [`RouteUri`] (`rocket::route::RouteUri`).

use napi_derive::napi;
use rocket::http::Method;
use rocket::route::dummy_handler;
use rocket::Route as RocketRoute;

/// NAPI mirror of Rocket's `RouteUri`.
#[napi]
pub struct RouteUri {
    pub base: String,
    pub unmounted_origin: String,
    pub origin: String,
    pub path: String,
    pub query: Option<String>,
}

#[napi]
impl RouteUri {
    /// Create a new `RouteUri` given a base and origin string.
    #[napi(constructor)]
    pub fn new(base: String, uri: String) -> Self {
        let route = RocketRoute::new(Method::Get, &uri, dummy_handler);
        let mapped = route
            .map_base(|_| base.clone())
            .unwrap_or_else(|_| RocketRoute::new(Method::Get, &uri, dummy_handler));

        let canonical_origin = mapped.uri.as_str().to_string();
        let path = canonical_origin.split('?').next().unwrap_or(&canonical_origin).to_string();
        let query = canonical_origin.split_once('?').map(|(_, q)| q.to_string());

        RouteUri {
            base: mapped.uri.base().to_string(),
            unmounted_origin: uri,
            origin: canonical_origin,
            path,
            query,
        }
    }

    #[napi]
    pub fn as_str(&self) -> String {
        self.origin.clone()
    }
}

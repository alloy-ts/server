//! NAPI bindings for Rocket's [`RouteUri`] (`rocket::route::RouteUri`).

use napi_derive::napi;

use rocket::http::Method;
use rocket::route::dummy_handler;
use rocket::route::Route as RocketRoute;

/// NAPI wrapper for Rocket's `RouteUri`.
#[napi]
pub struct RouteUri {
    pub base: String,
    pub path: String,
    pub query: Option<String>,
    pub as_str: String,
    pub default_rank: i64,
}

#[napi]
impl RouteUri {
    /// Creates a new `RouteUri` from `base` and `uri`.
    #[napi(constructor)]
    pub fn new(base: String, uri: String) -> napi::Result<Self> {
        let route = RocketRoute::new(Method::Get, &uri, dummy_handler);
        let route = if base != "/" {
            route
                .map_base(|_| base)
                .map_err(|e| napi::Error::new(napi::Status::GenericFailure, e.to_string()))?
        } else {
            route
        };

        Ok(RouteUri {
            base: route.uri.base().to_string(),
            path: route.uri.path().to_string(),
            query: route.uri.query().map(|q| q.to_string()),
            as_str: route.uri.as_str().to_string(),
            default_rank: route.rank as i64,
        })
    }
}

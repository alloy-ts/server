//! # NAPI bindings for Rocket's `RouteUri` structure.

use napi_derive::napi;
use rocket::http::Method;
use rocket::route::dummy_handler;
use rocket::Route;

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
    #[napi(constructor)]
    pub fn new(base: Option<String>, uri: String) -> napi::Result<Self> {
        let base_str = base.as_deref().unwrap_or("/");
        let route_res = std::panic::catch_unwind(|| {
            Route::new(Method::Get, &uri, dummy_handler)
        });
        let route = route_res.map_err(|_| {
            napi::Error::new(napi::Status::InvalidArg, format!("invalid route URI: {uri}"))
        })?;

        let route_uri = if base_str != "/" {
            let mapped_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                route.map_base(|_| base_str.to_string())
            }));
            match mapped_res {
                Ok(Ok(r)) => r.uri,
                _ => return Err(napi::Error::new(napi::Status::InvalidArg, format!("invalid base URI: {base_str}"))),
            }
        } else {
            route.uri
        };

        Ok(RouteUri {
            base: route_uri.base().to_string(),
            unmounted_origin: route_uri.unmounted_origin.to_string(),
            origin: route_uri.origin.to_string(),
            path: route_uri.path().to_string(),
            query: route_uri.query().map(|q| q.to_string()),
        })
    }

    #[napi]
    pub fn as_str(&self) -> String {
        self.origin.clone()
    }
}

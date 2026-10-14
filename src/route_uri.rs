//! NAPI bindings for Rocket's [`RouteUri`] (`rocket::route::RouteUri`).
//!
//! Represents a parsed route URI, containing its mount `base`, `unmountedOrigin`,
//! `origin` path, query parameters, and string representation.

use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
pub struct RouteUri {
    base: String,
    unmounted_origin: String,
    origin: String,
    path: String,
    query: Option<String>,
}

#[napi]
impl RouteUri {
    #[napi(constructor)]
    pub fn new(base: String, uri: String) -> napi::Result<Self> {
        let route = rocket::Route::ranked(
            None,
            rocket::http::Method::Get,
            &uri,
            rocket::route::dummy_handler,
        );
        let route_uri = match route.map_base(|_| base.clone()) {
            Ok(r) => r.uri,
            Err(_) => {
                return Err(Error::new(
                    Status::InvalidArg,
                    format!("invalid URI or base: base='{base}', uri='{uri}'"),
                ));
            }
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
    pub fn base(&self) -> String {
        self.base.clone()
    }

    #[napi]
    pub fn path(&self) -> String {
        self.path.clone()
    }

    #[napi]
    pub fn query(&self) -> Option<String> {
        self.query.clone()
    }

    #[napi]
    pub fn as_str(&self) -> String {
        self.origin.clone()
    }

    #[napi(getter)]
    pub fn unmounted_origin(&self) -> String {
        self.unmounted_origin.clone()
    }

    #[napi(getter)]
    pub fn origin(&self) -> String {
        self.origin.clone()
    }
}

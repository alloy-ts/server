//! NAPI bindings for Rocket's [`RouteUri`](rocket::route::RouteUri).

use napi_derive::napi;

use rocket::http::Method;
use rocket::route::dummy_handler;
use rocket::Route as RocketRoute;

#[napi]
pub struct RouteUri {
    pub base: String,
    pub unmounted_origin: String,
    pub origin: String,
}

#[napi]
impl RouteUri {
    #[napi(constructor)]
    pub fn new(base: String, origin: String) -> Self {
        let rocket_route = RocketRoute::new(Method::Get, &origin, dummy_handler);
        let mapped = rocket_route
            .clone()
            .map_base(|_| base.clone())
            .unwrap_or(rocket_route);

        Self {
            base: mapped.uri.base().to_string(),
            unmounted_origin: mapped.uri.path().to_string(),
            origin: mapped.uri.as_str().to_string(),
        }
    }

    #[napi]
    pub fn path(&self) -> String {
        self.origin.clone()
    }

    #[napi]
    pub fn query(&self) -> Option<String> {
        let rocket_route = RocketRoute::new(Method::Get, &self.origin, dummy_handler);
        rocket_route.uri.query().map(|q| q.to_string())
    }

    #[napi]
    pub fn as_str(&self) -> String {
        self.origin.clone()
    }
}

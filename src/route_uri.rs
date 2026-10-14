use napi::bindgen_prelude::*;
use napi_derive::napi;
use rocket::route::dummy_handler;
use rocket::Route;

/// NAPI wrapper around Rocket's `RouteUri` struct.
#[napi(js_name = "RouteUri")]
pub struct RouteUri {
    /// The mount point base path.
    pub base: String,
    /// The URI path without the base mount point.
    pub unmounted_origin: String,
    /// The canonical URI path with the base mount point.
    pub origin: String,
    path_str: String,
    query_str: Option<String>,
}

#[napi]
impl RouteUri {
    /// Creates a new RouteUri given a base mount path and URI origin string.
    #[napi(constructor)]
    pub fn new(base: String, uri: String) -> napi::Result<Self> {
        let route = Route::new(rocket::http::Method::Get, &uri, dummy_handler);
        let route = route
            .map_base(|_| base.clone())
            .map_err(|_| napi::Error::new(Status::InvalidArg, format!("Invalid base URI: '{base}'")))?;

        Ok(Self {
            base: route.uri.base().to_string(),
            unmounted_origin: route.uri.unmounted_origin.to_string(),
            origin: route.uri.origin.to_string(),
            path_str: route.uri.path().to_string(),
            query_str: route.uri.query().map(|q| q.to_string()),
        })
    }

    /// Parse a base and URI into a RouteUri.
    #[napi(factory)]
    pub fn parse(base: String, uri: String) -> napi::Result<Self> {
        Self::new(base, uri)
    }

    /// Returns the path part of the base mount point.
    #[napi(getter)]
    pub fn get_base(&self) -> String {
        self.base.clone()
    }

    /// Returns the path part of this route URI (excluding query string).
    #[napi(getter)]
    pub fn path(&self) -> String {
        self.path_str.clone()
    }

    /// Returns the query string part of this route URI, if any.
    #[napi(getter)]
    pub fn query(&self) -> Option<String> {
        self.query_str.clone()
    }

    /// Returns the full canonical URI string.
    #[napi]
    pub fn as_str(&self) -> String {
        self.origin.clone()
    }
}

//! NAPI bindings for Rocket's [`Route`] (`rocket::Route`).
//!
//! A `Route` represents a request-handling route containing a method, URI,
//! rank, format, and optional name metadata.

use napi_derive::napi;

/// NAPI object representation of Rocket's `Route`.
#[napi(object)]
pub struct Route {
    pub name: Option<String>,
    pub method: String,
    pub uri: String,
    pub rank: i64,
    pub format: Option<String>,
}

/// Creates a new `Route` metadata object, computing default rank if omitted.
#[napi]
pub fn create_route(
    method: String,
    uri: String,
    name: Option<String>,
    rank: Option<i64>,
    format: Option<String>,
) -> Route {
    Route {
        name,
        method: method.to_uppercase(),
        uri,
        rank: rank.unwrap_or(-9),
        format,
    }
}

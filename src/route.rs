use napi_derive::napi;
use rocket::http::{MediaType, Method};
use rocket::route::dummy_handler;
use rocket::Route as RocketRoute;
use std::str::FromStr;

/// NAPI wrapper around Rocket's `Route` struct.
#[napi(js_name = "Route")]
pub struct Route {
    /// The name of this route, if one was given.
    pub name: Option<String>,
    /// The HTTP method this route matches against.
    pub method: String,
    /// The route URI path.
    pub uri: String,
    /// The rank of this route (lower numbers mean higher precedence).
    pub rank: i32,
    /// The media type format this route matches against (e.g. "json", "html").
    pub format: Option<String>,
}

#[napi]
impl Route {
    /// Creates a new Route with method, URI, and optional rank / format.
    #[napi(constructor)]
    pub fn new(
        method: String,
        uri: String,
        rank: Option<i32>,
        format: Option<String>,
    ) -> napi::Result<Self> {
        let parsed_method = Method::from_str(&method).unwrap_or(Method::Get);

        let rocket_route = if let Some(r) = rank {
            RocketRoute::ranked(Some(r as isize), parsed_method, &uri, dummy_handler)
        } else {
            RocketRoute::new(parsed_method, &uri, dummy_handler)
        };

        let mut route = Route {
            name: None,
            method: parsed_method.as_str().to_string(),
            uri: rocket_route.uri.to_string(),
            rank: rocket_route.rank as i32,
            format: None,
        };

        if let Some(fmt) = format {
            if let Some(media_type) = MediaType::from_extension(&fmt) {
                route.format = Some(media_type.to_string());
            } else {
                route.format = Some(fmt);
            }
        }

        Ok(route)
    }

    /// Creates a Route with default computed rank.
    #[napi(factory)]
    pub fn create(method: String, uri: String) -> napi::Result<Self> {
        Self::new(method, uri, None, None)
    }

    /// Creates a Route with a specific rank.
    #[napi(factory)]
    pub fn ranked(method: String, uri: String, rank: i32) -> napi::Result<Self> {
        Self::new(method, uri, Some(rank), None)
    }

    /// Calculates Rocket's default rank for a given URI.
    ///
    /// Table:
    /// static path, static query -> -12
    /// static path, partial query -> -11
    /// static path, wild query    -> -10
    /// static path, no query      -> -9
    /// partial path, static query -> -8
    /// partial path, partial query-> -7
    /// partial path, wild query   -> -6
    /// partial path, no query     -> -5
    /// wild path, static query    -> -4
    /// wild path, partial query   -> -3
    /// wild path, wild query      -> -2
    /// wild path, no query        -> -1
    #[napi]
    pub fn default_rank(uri: String) -> i32 {
        let route = RocketRoute::new(Method::Get, &uri, dummy_handler);
        route.rank as i32
    }
}

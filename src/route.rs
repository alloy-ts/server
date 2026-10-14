//! # NAPI bindings for Rocket's `Route` structure.
//!
//! Exposes Rocket's `Route` metadata and constructor methods (`new`, `ranked`)
//! to TypeScript, mirroring `rocket::Route`.

use napi_derive::napi;
use rocket::http::Method as RocketMethod;
use rocket::route::dummy_handler;
use rocket::Route as RocketRoute;

#[napi]
pub struct Route {
    pub name: Option<String>,
    pub method: String,
    pub uri: String,
    pub rank: i64,
    pub format: Option<String>,
}

#[napi]
impl Route {
    /// Creates a new route with the given method, path, and default computed rank.
    #[napi(constructor)]
    pub fn new(method: String, uri: String, name: Option<String>, format: Option<String>) -> napi::Result<Self> {
        let rocket_method: RocketMethod = method
            .parse()
            .map_err(|_| napi::Error::new(napi::Status::GenericFailure, format!("invalid HTTP method: {method}")))?;

        let mut route = RocketRoute::new(rocket_method, &uri, dummy_handler);
        if let Some(n) = name {
            route.name = Some(n.into());
        }
        if let Some(f) = format {
            if let Ok(media_type) = f.parse() {
                route.format = Some(media_type);
            }
        }

        Ok(Route {
            name: route.name.map(|n| n.to_string()),
            method: route.method.as_str().to_string(),
            uri: route.uri.to_string(),
            rank: route.rank as i64,
            format: route.format.map(|f| f.to_string()),
        })
    }

    /// Creates a route with an explicit rank override.
    #[napi(factory)]
    pub fn ranked(rank: Option<i64>, method: String, uri: String, name: Option<String>, format: Option<String>) -> napi::Result<Self> {
        let rocket_method: RocketMethod = method
            .parse()
            .map_err(|_| napi::Error::new(napi::Status::GenericFailure, format!("invalid HTTP method: {method}")))?;

        let rank_opt = rank.map(|r| r as isize);
        let mut route = RocketRoute::ranked(rank_opt, rocket_method, &uri, dummy_handler);
        if let Some(n) = name {
            route.name = Some(n.into());
        }
        if let Some(f) = format {
            if let Ok(media_type) = f.parse() {
                route.format = Some(media_type);
            }
        }

        Ok(Route {
            name: route.name.map(|n| n.to_string()),
            method: route.method.as_str().to_string(),
            uri: route.uri.to_string(),
            rank: route.rank as i64,
            format: route.format.map(|f| f.to_string()),
        })
    }
}

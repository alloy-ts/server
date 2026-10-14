//! NAPI bindings for Rocket's [`Route`](rocket::Route).
//!
//! A request handling route metadata wrapper matching Rocket's `Route` struct.

use napi::bindgen_prelude::*;
use napi_derive::napi;

use rocket::http::Method;
use rocket::route::dummy_handler;
use rocket::Route as RocketRoute;

#[napi]
#[derive(Clone)]
pub struct Route {
    pub name: Option<String>,
    pub method: String,
    pub uri: String,
    pub rank: i64,
    pub format: Option<String>,
}

#[napi]
impl Route {
    #[napi(constructor)]
    pub fn new(
        method: String,
        uri: String,
        name: Option<String>,
        rank: Option<i64>,
        format: Option<String>,
    ) -> napi::Result<Self> {
        let m: Method = method
            .parse()
            .map_err(|_| napi::Error::new(Status::GenericFailure, format!("invalid method: {method}")))?;

        let rank_opt = rank.map(|r| r as isize);
        let rocket_route = RocketRoute::ranked(rank_opt, m, &uri, dummy_handler);

        Ok(Self {
            name,
            method: rocket_route.method.as_str().to_string(),
            uri: rocket_route.uri.to_string(),
            rank: rocket_route.rank as i64,
            format,
        })
    }

    #[napi]
    pub fn ranked(rank: Option<i64>, method: String, uri: String) -> napi::Result<Self> {
        Self::new(method, uri, None, rank, None)
    }

    #[napi]
    pub fn map_base(&mut self, base_prefix: String) {
        if !base_prefix.is_empty() && base_prefix != "/" {
            let clean_prefix = base_prefix.trim_end_matches('/');
            self.uri = format!("{clean_prefix}{}", self.uri);
        }
    }
}

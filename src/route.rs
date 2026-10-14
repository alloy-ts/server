//! NAPI bindings for Rocket's [`Route`] (`rocket::Route`).

use napi::bindgen_prelude::Error as NapiError;
use napi::Status;
use napi_derive::napi;

use rocket::http::{MediaType, Method};
use rocket::route::{dummy_handler, Route as RocketRoute};

fn err<E: std::fmt::Display>(e: E) -> NapiError {
    NapiError::new(Status::GenericFailure, e.to_string())
}

/// NAPI binding for Rocket's [`Route`].
#[napi]
#[derive(Clone)]
pub struct Route {
    pub name: Option<String>,
    pub method: String,
    pub uri: String,
    pub rank: i64,
    pub format: Option<String>,
    #[napi(skip)]
    pub inner: RocketRoute,
}

#[napi]
impl Route {
    /// Creates a new route with the given method, path, and default rank.
    #[napi(constructor)]
    pub fn new(
        method: String,
        uri: String,
        name: Option<String>,
        format: Option<String>,
    ) -> napi::Result<Self> {
        Self::ranked(None, method, uri, name, format)
    }

    /// Creates a new route with an optional explicit rank, method, path, name, and format.
    #[napi]
    pub fn ranked(
        rank: Option<i64>,
        method: String,
        uri: String,
        name: Option<String>,
        format: Option<String>,
    ) -> napi::Result<Self> {
        let m: Method = method
            .parse()
            .map_err(|_| NapiError::new(Status::GenericFailure, format!("invalid method: {method}")))?;

        let rank_option = rank.map(|r| r as isize);
        let mut rocket_route = RocketRoute::ranked(rank_option, m, &uri, dummy_handler);

        if let Some(ref n) = name {
            rocket_route.name = Some(std::borrow::Cow::Owned(n.clone()));
        }

        if let Some(ref fmt) = format {
            if let Some(media_type) = MediaType::parse_flexible(fmt) {
                rocket_route.format = Some(media_type);
            }
        }

        let computed_rank = rocket_route.rank as i64;
        let method_str = rocket_route.method.as_str().to_string();
        let uri_str = rocket_route.uri.to_string();
        let name_str = rocket_route.name.as_ref().map(|n| n.to_string());
        let format_str = rocket_route.format.as_ref().map(|f| f.to_string());

        Ok(Route {
            name: name_str,
            method: method_str,
            uri: uri_str,
            rank: computed_rank,
            format: format_str,
            inner: rocket_route,
        })
    }

    /// Maps the base mount point of this route.
    #[napi]
    pub fn map_base(&mut self, base: String) -> napi::Result<String> {
        let current_route = self.inner.clone();
        let mapped = current_route
            .map_base(|_curr| base.clone())
            .map_err(err)?;
        self.uri = mapped.uri.to_string();
        self.inner = mapped;
        Ok(self.uri.clone())
    }
}

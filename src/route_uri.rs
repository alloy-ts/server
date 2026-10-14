//! NAPI bindings for Rocket's [`RouteUri`](rocket::route::RouteUri).

use napi::bindgen_prelude::Error as NapiError;
use napi::Status;
use napi_derive::napi;

use rocket::http::Method;
use rocket::route::{dummy_handler, Route as RocketRoute, RouteUri as RocketRouteUri};

fn err<E: std::fmt::Display>(e: E) -> NapiError {
    NapiError::new(Status::GenericFailure, e.to_string())
}

/// NAPI binding for Rocket's [`RouteUri`].
#[napi]
#[derive(Clone)]
pub struct RouteUri {
    pub base: String,
    pub unmounted_origin: String,
    pub origin: String,
    pub default_rank: i64,
    #[napi(skip)]
    pub inner: RocketRouteUri<'static>,
}

#[napi]
impl RouteUri {
    /// Creates a new `RouteUri` with the given base mount point and URI.
    #[napi(constructor)]
    pub fn new(base: Option<String>, uri: String) -> napi::Result<Self> {
        let base_str = base.as_deref().unwrap_or("/");
        let mut route = RocketRoute::ranked(None, Method::Get, &uri, dummy_handler);
        if base_str != "/" {
            route = route.map_base(|_| base_str.to_string()).map_err(err)?;
        }
        let inner = route.uri;
        let default_rank = route.rank as i64;

        Ok(RouteUri {
            base: inner.base().to_string(),
            unmounted_origin: inner.unmounted_origin.to_string(),
            origin: inner.origin.to_string(),
            default_rank,
            inner,
        })
    }

    /// The path part of this route URI.
    #[napi]
    pub fn path(&self) -> String {
        self.inner.path().to_string()
    }

    /// The query part of this route URI, if present.
    #[napi]
    pub fn query(&self) -> Option<String> {
        self.inner.query().map(|q| q.to_string())
    }

    /// The full URI as a string.
    #[napi]
    pub fn as_str(&self) -> String {
        self.inner.as_str().to_string()
    }
}

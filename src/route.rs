//! NAPI bindings for Rocket's [`Route`](rocket::Route) type.
//!
//! A [`Route`] consists of a method, URI pattern, rank, format, optional name,
//! and request [`Handler`](rocket::route::Handler).

use napi::bindgen_prelude::*;
use napi_derive::napi;

use rocket::http::Method;
use rocket::route::dummy_handler;
use rocket::Route as RocketRoute;

use crate::handler::HandlerBridge;

/// A NAPI wrapper around Rocket's [`Route`](rocket::Route).
#[napi]
#[derive(Clone)]
pub struct Route {
    pub(crate) inner: RocketRoute,
}

#[napi]
impl Route {
    /// Create a new `Route` with default ranking.
    #[napi(constructor)]
    #[allow(deprecated)]
    pub fn new(method: String, uri: String, handler: Option<Unknown>) -> Result<Self> {
        let m: Method = method
            .parse()
            .map_err(|_| Error::new(Status::GenericFailure, format!("invalid method: {method}")))?;

        let route = if let Some(h) = handler {
            let bridge = HandlerBridge::new(h)?;
            RocketRoute::new(m, &uri, bridge)
        } else {
            RocketRoute::new(m, &uri, dummy_handler)
        };

        Ok(Self { inner: route })
    }

    /// Create a new ranked `Route`.
    #[napi]
    #[allow(deprecated)]
    pub fn ranked(
        rank: Option<i32>,
        method: String,
        uri: String,
        handler: Option<Unknown>,
    ) -> Result<Self> {
        let m: Method = method
            .parse()
            .map_err(|_| Error::new(Status::GenericFailure, format!("invalid method: {method}")))?;

        let r = rank.map(|r| r as isize);

        let route = if let Some(h) = handler {
            let bridge = HandlerBridge::new(h)?;
            RocketRoute::ranked(r, m, &uri, bridge)
        } else {
            RocketRoute::ranked(r, m, &uri, dummy_handler)
        };

        Ok(Self { inner: route })
    }

    #[napi(getter)]
    pub fn name(&self) -> Option<String> {
        self.inner.name.as_ref().map(|n| n.to_string())
    }

    #[napi(getter)]
    pub fn method(&self) -> String {
        self.inner.method.as_str().to_string()
    }

    #[napi(getter)]
    pub fn uri(&self) -> String {
        self.inner.uri.to_string()
    }

    #[napi(getter)]
    pub fn rank(&self) -> i32 {
        self.inner.rank as i32
    }

    #[napi(getter)]
    pub fn format(&self) -> Option<String> {
        self.inner.format.as_ref().map(|f| f.to_string())
    }

    /// Map the base mount path of this route, returning a new `Route`.
    #[napi]
    pub fn map_base(&self, base: String) -> Result<Self> {
        let mut cloned = self.inner.clone();
        cloned = cloned
            .map_base(|_| base)
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
        Ok(Self { inner: cloned })
    }
}

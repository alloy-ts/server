//! NAPI bindings for Service's `Route` struct.
//!
//! A Service `Route` pairs a `Method` + URI pattern with a
//! [`Handler`](crate::handler::Handler). Service normally generates routes via
//! the `#[get]`/etc. attributes; this module exposes the equivalent at runtime
//! so TypeScript can build and inspect routes directly:
//!
//! * [`Route`] — a `#[napi]` class wrapping `service::route::Route`. Construct it
//!   with [`Route::new`] (default rank) or [`Route::ranked`] (explicit rank),
//!   each taking a method, a URI pattern, and a JS handler function. The pattern
//!   may be Service syntax (`<name>`) or Bun/Express syntax (`:name`, `*`) — it
//!   is translated the same way as the `serve` route table.
//! * Field getters mirror the Rust fields: `name`, `method`, `uri`, `rank`,
//!   `format`.
//! * [`Route::map_base`] wraps `Route::map_base`, and [`Route::register`] appends
//!   the route to the shared registry that [`serve`](crate::service::serve)
//!   mounts (call `register` before `serve`).
//!
//! A pre-built [`Handler`](crate::handler::Handler) instance can be attached
//! with [`Route::from_handler`].

use napi::bindgen_prelude::{Error, Status};
use napi::Unknown;
use napi_derive::napi;
use service::http::Method;
use service::route::Route as ServiceRoute;

use crate::handler::Handler;
use crate::service::{build_route, routes};

/// A NAPI wrapper around `service::route::Route`.
///
/// Holds the real Service `Route`; the `handler` inside is always a JS-backed
/// `Handler` (`crate::handler::Handler`) wrapped as a `service::route::Handler`.
#[napi]
pub struct Route {
    inner: ServiceRoute,
}

impl Route {
    /// Consume this NAPI `Route`, returning the underlying `service::Route`.
    pub(crate) fn into_inner(self) -> ServiceRoute {
        self.inner
    }
}

#[napi]
impl Route {
    /// Build a route with a computed default rank.
    ///
    /// `handler` is a JS function `(req: Request) => Promise<Outcome>`; it
    /// is wrapped in a [`Handler`](crate::handler::Handler) automatically.
    #[napi(constructor)]
    pub fn new(method: String, uri: String, handler: Unknown) -> napi::Result<Self> {
        let m = parse_method(&method)?;
        let h = Handler::new(handler)?;
        let inner = build_route(m, &uri, None, &h)?;
        Ok(Route { inner })
    }

    /// Build a route with an explicit rank. Lower ranks have higher precedence.
    #[napi]
    pub fn ranked(rank: i32, method: String, uri: String, handler: Unknown) -> napi::Result<Self> {
        let m = parse_method(&method)?;
        let h = Handler::new(handler)?;
        let inner = build_route(m, &uri, Some(rank as isize), &h)?;
        Ok(Route { inner })
    }

    /// Build a route from an existing [`Handler`] instance (e.g. a stateful
    /// custom handler created with `new Handler(...)`).
    #[napi]
    pub fn from_handler(method: String, uri: String, handler: &Handler) -> napi::Result<Self> {
        let m = parse_method(&method)?;
        let inner = build_route(m, &uri, None, handler)?;
        Ok(Route { inner })
    }

    /// The route's name, if one was given.
    #[napi(getter)]
    pub fn name(&self) -> Option<String> {
        self.inner.name.as_ref().map(|c| c.to_string())
    }

    /// The HTTP method this route matches against.
    #[napi(getter)]
    pub fn method(&self) -> String {
        self.inner.method.as_str().to_string()
    }

    /// The route URI (base + unmounted origin).
    #[napi(getter)]
    pub fn uri(&self) -> String {
        self.inner.uri.as_str().to_string()
    }

    /// The route's rank. Lower ranks have higher precedence.
    #[napi(getter)]
    pub fn rank(&self) -> i32 {
        self.inner.rank as i32
    }

    /// The media type this route matches against, if any.
    #[napi(getter)]
    pub fn format(&self) -> Option<String> {
        self.inner.format.as_ref().map(|m| m.to_string())
    }

    /// Map the base of this route, returning a new `Route`. Mirrors
    /// `service::route::Route::map_base`.
    #[napi]
    pub fn map_base(&self, base: String) -> napi::Result<Route> {
        let mapped = self
            .inner
            .clone()
            .map_base(|_| base.clone())
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
        Ok(Route { inner: mapped })
    }

    /// Register this route on the shared registry that
    /// [`serve`](crate::service::serve) mounts. Call before `serve`.
    #[napi]
    pub fn register(&self) -> napi::Result<()> {
        routes().lock().unwrap().push(self.inner.clone());
        Ok(())
    }
}

/// Parse an HTTP method string, mapping errors to `napi::Error`.
fn parse_method(method: &str) -> napi::Result<Method> {
    method
        .parse()
        .map_err(|_| Error::new(Status::GenericFailure, format!("invalid method: {method}")))
}

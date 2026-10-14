//! NAPI bindings for Rocket's [`RouteUri`] (`rocket::route::RouteUri`).
//!
//! A route URI matched against requests, consisting of base mount point,
//! unmounted origin, and canonical origin URI.

use napi_derive::napi;
use rocket::http::Method;
use rocket::route::dummy_handler;
use rocket::Route as RocketRoute;

#[napi]
pub struct RouteUri {
  base: String,
  unmounted_origin: String,
  origin: String,
}

#[napi]
impl RouteUri {
  /// Create a new RouteUri given a base mount point and relative URI.
  #[napi(constructor)]
  pub fn new(base: String, uri: String) -> napi::Result<Self> {
    let r = RocketRoute::new(Method::Get, &uri, dummy_handler);
    let route = if base != "/" && !base.is_empty() {
      r.map_base(|_b| base.clone())
        .map_err(|e| napi::Error::new(napi::Status::GenericFailure, e.to_string()))?
    } else {
      r
    };
    Ok(RouteUri {
      base: route.uri.base().to_string(),
      unmounted_origin: route.uri.unmounted_origin.to_string(),
      origin: route.uri.as_str().to_string(),
    })
  }

  /// Returns the base mount point.
  #[napi]
  pub fn base(&self) -> String {
    self.base.clone()
  }

  /// Returns the URI without the base mount point.
  #[napi(js_name = "unmountedOrigin")]
  pub fn unmounted_origin(&self) -> String {
    self.unmounted_origin.clone()
  }

  /// Returns the path part of this route URI.
  #[napi]
  pub fn path(&self) -> String {
    self.origin.split('?').next().unwrap_or(&self.origin).to_string()
  }

  /// Returns the query part of this route URI if present.
  #[napi]
  pub fn query(&self) -> Option<String> {
    self.origin.split_once('?').map(|(_, q)| q.to_string())
  }

  /// Returns the full URI string representation.
  #[napi(js_name = "asStr")]
  pub fn as_str(&self) -> String {
    self.origin.clone()
  }
}

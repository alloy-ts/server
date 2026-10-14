//! # NAPI bindings for Rocket's [`Route`] (`rocket::Route`).
//!
//! A route consists of a method, URI pattern, rank, optional format (media type),
//! and handler procedure.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use rocket::http::Method;
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
  /// Creates a new route with the given method and path with a computed default rank.
  #[napi(constructor)]
  pub fn new(method: String, uri: String) -> napi::Result<Self> {
    let m: Method = method
      .parse()
      .map_err(|_| napi::Error::new(Status::GenericFailure, format!("invalid method: {method}")))?;
    let r = RocketRoute::new(m, &uri, dummy_handler);
    Ok(Route {
      name: r.name.map(|n| n.to_string()),
      method: r.method.as_str().to_string(),
      uri: r.uri.to_string(),
      rank: r.rank as i64,
      format: r.format.map(|f| f.to_string()),
    })
  }

  /// Creates a new route with the given rank, method, and path.
  #[napi(factory)]
  pub fn ranked(rank: Option<i64>, method: String, uri: String) -> napi::Result<Self> {
    let m: Method = method
      .parse()
      .map_err(|_| napi::Error::new(Status::GenericFailure, format!("invalid method: {method}")))?;
    let r = RocketRoute::ranked(rank.map(|r| r as isize), m, &uri, dummy_handler);
    Ok(Route {
      name: r.name.map(|n| n.to_string()),
      method: r.method.as_str().to_string(),
      uri: r.uri.to_string(),
      rank: r.rank as i64,
      format: r.format.map(|f| f.to_string()),
    })
  }
}

//! NAPI bindings for Rocket's [`Route`] (`rocket::Route`).
//!
//! Exposes route metadata and creation helpers (`name`, `method`, `uri`,
//! `rank`, `format`) to Node.js/TypeScript.

use napi::bindgen_prelude::Error as NapiError;
use napi::Status;
use napi_derive::napi;

use rocket::http::Method;

/// NAPI mirror of Rocket's `Route` metadata.
#[napi]
#[derive(Clone, Debug)]
pub struct Route {
  pub name: Option<String>,
  pub method: String,
  pub uri: String,
  pub rank: i64,
  pub format: Option<String>,
}

#[napi]
impl Route {
  /// Create a new `Route` with default rank and base `/`.
  #[napi(constructor)]
  pub fn new(method: String, uri: String) -> napi::Result<Self> {
    Self::ranked(None, method, uri)
  }

  /// Create a new `Route` with explicit rank.
  #[napi]
  pub fn ranked(rank: Option<i64>, method: String, uri: String) -> napi::Result<Self> {
    let m: Method = method
      .parse()
      .map_err(|_| NapiError::new(Status::InvalidArg, format!("invalid method: {method}")))?;

    let rank_isize = rank.map(|r| r as isize);
    let rocket_route = rocket::Route::ranked(rank_isize, m, &uri, rocket::route::dummy_handler);

    Ok(Route {
      name: rocket_route.name.map(|n| n.to_string()),
      method: rocket_route.method.as_str().to_string(),
      uri: rocket_route.uri.to_string(),
      rank: rocket_route.rank as i64,
      format: rocket_route.format.map(|f| f.to_string()),
    })
  }

  #[napi(getter)]
  pub fn get_name(&self) -> Option<String> {
    self.name.clone()
  }

  #[napi(getter)]
  pub fn get_method(&self) -> String {
    self.method.clone()
  }

  #[napi(getter)]
  pub fn get_uri(&self) -> String {
    self.uri.clone()
  }

  #[napi(getter)]
  pub fn get_rank(&self) -> i64 {
    self.rank
  }

  #[napi(getter)]
  pub fn get_format(&self) -> Option<String> {
    self.format.clone()
  }
}

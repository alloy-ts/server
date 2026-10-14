//! # Rocket - RouteUri NAPI Bindings
//!
//! NAPI bindings for Rocket's `RouteUri` struct.

use napi_derive::napi;

/// A route URI which is matched against requests.
#[napi]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RouteUri {
  /// The base mount point (e.g. "/base" or "/").
  pub base: String,
  /// The URI without the base mount point.
  pub unmounted_origin: String,
  /// The canonical URI with base mount point.
  pub origin: String,
}

#[napi]
impl RouteUri {
  #[napi(constructor)]
  pub fn new(base: Option<String>, uri: String) -> Self {
    let base = base.unwrap_or_else(|| "/".to_string());
    let norm_base = if base.starts_with('/') {
      base
    } else {
      format!("/{base}")
    };
    let norm_base = norm_base.trim_end_matches('/').to_string();
    let norm_base = if norm_base.is_empty() {
      "/".to_string()
    } else {
      norm_base
    };

    let norm_uri = if uri.starts_with('/') {
      uri.clone()
    } else {
      format!("/{uri}")
    };

    let canonical = if norm_base == "/" {
      norm_uri.clone()
    } else {
      format!("{}{}", norm_base, norm_uri)
    };

    RouteUri {
      base: norm_base,
      unmounted_origin: norm_uri,
      origin: canonical,
    }
  }

  /// Returns the base mount point.
  #[napi]
  pub fn base(&self) -> String {
    self.base.clone()
  }

  /// Returns the path part of this route URI.
  #[napi]
  pub fn path(&self) -> String {
    self
      .origin
      .split('?')
      .next()
      .unwrap_or(&self.origin)
      .to_string()
  }

  /// Returns the query part of this route URI, if any.
  #[napi]
  pub fn query(&self) -> Option<String> {
    self.origin.split_once('?').map(|(_, q)| q.to_string())
  }

  /// Returns the full canonical URI as string.
  #[napi]
  pub fn as_str(&self) -> String {
    self.origin.clone()
  }

  /// Returns the default rank of a route with this URI according to Rocket's default ranking table.
  #[napi]
  pub fn default_rank(&self) -> i32 {
    let path = self.path();
    let query = self.query();

    let path_color = path_color(&path);
    let query_color = query_color(query.as_deref());

    let raw_path_weight = path_color as u8;
    let raw_query_weight = query_color.map_or(0, |c| c as u8);
    let raw_weight = (raw_path_weight << 2) | raw_query_weight;

    -((raw_weight as i32) - 3)
  }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Color {
  Static = 3,
  Partial = 2,
  Wild = 1,
}

fn path_color(path: &str) -> Color {
  let segs: Vec<&str> = path
    .trim_start_matches('/')
    .split('/')
    .filter(|s| !s.is_empty())
    .collect();

  if segs.is_empty() {
    return Color::Static;
  }

  let static_count = segs.iter().filter(|s| !s.starts_with('<')).count();
  let dyn_count = segs.iter().filter(|s| s.starts_with('<')).count();

  if static_count == segs.len() {
    Color::Static
  } else if dyn_count == segs.len() {
    Color::Wild
  } else {
    Color::Partial
  }
}

fn query_color(query: Option<&str>) -> Option<Color> {
  let q = query?;
  if q.is_empty() {
    return None;
  }

  let segs: Vec<&str> = q.split('&').filter(|s| !s.is_empty()).collect();
  if segs.is_empty() {
    return None;
  }

  let static_count = segs.iter().filter(|s| !s.contains('<')).count();
  let dyn_count = segs.iter().filter(|s| s.contains('<')).count();

  if static_count == segs.len() {
    Some(Color::Static)
  } else if dyn_count == segs.len() {
    Some(Color::Wild)
  } else {
    Some(Color::Partial)
  }
}

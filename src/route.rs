//! NAPI bindings for Rocket's [`Route`] (`rocket::Route`).
//!
//! Rocket routes represent a request-handling endpoint with a method, URI, rank,
//! format (media type), and handler. This module provides a NAPI mirror of
//! Rocket's `Route` structure and its default ranking logic.

use napi_derive::napi;

/// Compute Rocket's default rank for a given route URI.
///
/// Rocket ranks routes based on path and query specificity:
/// - Path colors: static, partial (<p>), wild (<p..>)
/// - Query colors: static, partial, wild, none
///
/// Default rank table:
/// | path    | query   | rank |
/// | static  | static  | -12  |
/// | static  | partial | -11  |
/// | static  | wild    | -10  |
/// | static  | none    | -9   |
/// | partial | static  | -8   |
/// | partial | partial | -7   |
/// | partial | wild    | -6   |
/// | partial | none    | -5   |
/// | wild    | static  | -4   |
/// | wild    | partial | -3   |
/// | wild    | wild    | -2   |
/// | wild    | none    | -1   |
#[napi]
pub fn compute_default_rank(uri: String) -> i32 {
    let (path_part, query_part) = uri.split_once('?').map_or((uri.as_str(), None), |(p, q)| (p, Some(q)));

    let path_color = get_path_color(path_part);
    let query_color = query_part.map_or(QueryColor::None, get_query_color);

    match (path_color, query_color) {
        (PathColor::Static, QueryColor::Static) => -12,
        (PathColor::Static, QueryColor::Partial) => -11,
        (PathColor::Static, QueryColor::Wild) => -10,
        (PathColor::Static, QueryColor::None) => -9,
        (PathColor::Partial, QueryColor::Static) => -8,
        (PathColor::Partial, QueryColor::Partial) => -7,
        (PathColor::Partial, QueryColor::Wild) => -6,
        (PathColor::Partial, QueryColor::None) => -5,
        (PathColor::Wild, QueryColor::Static) => -4,
        (PathColor::Wild, QueryColor::Partial) => -3,
        (PathColor::Wild, QueryColor::Wild) => -2,
        (PathColor::Wild, QueryColor::None) => -1,
    }
}

enum PathColor {
    Static,
    Partial,
    Wild,
}

enum QueryColor {
    Static,
    Partial,
    Wild,
    None,
}

fn get_path_color(path: &str) -> PathColor {
    let segs: Vec<&str> = path.trim_start_matches('/').split('/').filter(|s| !s.is_empty()).collect();
    if segs.is_empty() {
        return PathColor::Static;
    }

    let mut dynamic_count = 0;
    for seg in &segs {
        if seg.starts_with('<') && seg.ends_with('>') {
            dynamic_count += 1;
        }
    }

    if dynamic_count == 0 {
        PathColor::Static
    } else if dynamic_count == segs.len() {
        PathColor::Wild
    } else {
        PathColor::Partial
    }
}

fn get_query_color(query: &str) -> QueryColor {
    let segs: Vec<&str> = query.split('&').filter(|s| !s.is_empty()).collect();
    if segs.is_empty() {
        return QueryColor::None;
    }

    let mut dynamic_count = 0;
    for seg in &segs {
        if seg.contains('<') && seg.contains('>') {
            dynamic_count += 1;
        }
    }

    if dynamic_count == 0 {
        QueryColor::Static
    } else if dynamic_count == segs.len() {
        QueryColor::Wild
    } else {
        QueryColor::Partial
    }
}

/// NAPI wrapper class for Rocket's `Route`.
#[napi]
#[derive(Clone)]
pub struct Route {
    name: Option<String>,
    method: String,
    uri: String,
    rank: i32,
    format: Option<String>,
}

#[napi]
impl Route {
    /// Creates a new route with default computed rank.
    #[napi(constructor)]
    pub fn new(method: String, uri: String, name: Option<String>, format: Option<String>) -> Self {
        let rank = compute_default_rank(uri.clone());
        Route {
            name,
            method: method.to_uppercase(),
            uri,
            rank,
            format,
        }
    }

    /// Creates a new route with an explicit rank (or computed rank if rank is null/undefined).
    #[napi]
    pub fn ranked(
        rank: Option<i32>,
        method: String,
        uri: String,
        name: Option<String>,
        format: Option<String>,
    ) -> Self {
        let actual_rank = rank.unwrap_or_else(|| compute_default_rank(uri.clone()));
        Route {
            name,
            method: method.to_uppercase(),
            uri,
            rank: actual_rank,
            format,
        }
    }

    #[napi(getter)]
    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }

    #[napi(getter)]
    pub fn method(&self) -> String {
        self.method.clone()
    }

    #[napi(getter)]
    pub fn uri(&self) -> String {
        self.uri.clone()
    }

    #[napi(getter)]
    pub fn rank(&self) -> i32 {
        self.rank
    }

    #[napi(getter)]
    pub fn format(&self) -> Option<String> {
        self.format.clone()
    }
}

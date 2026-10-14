//! # Rocket - Route NAPI Bindings
//!
//! This module implements NAPI bindings for Rocket's [`Route`] metadata and routing representation.
//!
//! A [`Route`] in Rocket consists of:
//! - `name`: An optional identifier (e.g. function name in macro codegen).
//! - `method`: HTTP method (`GET`, `POST`, `PUT`, `DELETE`, `PATCH`, `HEAD`, `OPTIONS`).
//! - `uri`: The route pattern URI (e.g. `/users/<id>?<query>`).
//! - `rank`: The precedence integer (lower rank = higher priority).
//! - `format`: Optional media type / Content-Type / Accept matching format.
//!
//! Rocket automatically computes default ranks based on the static/dynamic components of paths and queries:
//! - Static path + static query -> rank -12
//! - Static path + no query -> rank -9
//! - Partial path + no query -> rank -5
//! - Wild path + no query -> rank -1

use napi_derive::napi;

/// NAPI object representing route metadata in Rocket.
#[napi(object)]
#[derive(Debug, Clone)]
pub struct RouteInfo {
    /// The name of this route, if one was provided.
    pub name: Option<String>,
    /// The HTTP method this route matches against.
    pub method: String,
    /// The route pattern URI.
    pub uri: String,
    /// The priority rank of this route (lower numbers = higher priority).
    pub rank: i32,
    /// Optional media type format (e.g., "json", "application/json").
    pub format: Option<String>,
}

/// Helper function to compute Rocket's default rank for a route URI pattern.
#[napi]
pub fn compute_default_rank(uri: String) -> i32 {
    let parts: Vec<&str> = uri.splitn(2, '?').collect();
    let path = parts[0];
    let query = parts.get(1);

    let path_segs: Vec<&str> = path
        .trim_start_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();

    let mut p_has_static = false;
    let mut p_has_dynamic = false;

    if path_segs.is_empty() {
        p_has_static = true;
    } else {
        for seg in path_segs {
            if seg.contains('<') || seg.contains(':') || seg.contains('*') {
                p_has_dynamic = true;
            } else {
                p_has_static = true;
            }
        }
    }

    let is_path_static = p_has_static && !p_has_dynamic;
    let is_path_partial = p_has_static && p_has_dynamic;

    let (is_q_static, is_q_partial, is_q_wild) = match query {
        None => (false, false, false),
        Some(q) => {
            let segs: Vec<&str> = q.split('&').collect();
            let mut has_static = false;
            let mut has_dynamic = false;
            for seg in segs {
                if seg.contains('<') || seg.contains(':') || seg.contains('*') {
                    has_dynamic = true;
                } else if !seg.is_empty() {
                    has_static = true;
                }
            }
            if has_static && has_dynamic {
                (false, true, false)
            } else if has_dynamic {
                (false, false, true)
            } else {
                (true, false, false)
            }
        }
    };

    if is_path_static {
        if is_q_static {
            -12
        } else if is_q_partial {
            -11
        } else if is_q_wild {
            -10
        } else {
            -9
        }
    } else if is_path_partial {
        if is_q_static {
            -8
        } else if is_q_partial {
            -7
        } else if is_q_wild {
            -6
        } else {
            -5
        }
    } else if is_q_static {
        -4
    } else if is_q_partial {
        -3
    } else if is_q_wild {
        -2
    } else {
        -1
    }
}

/// Construct a `RouteInfo` object with auto-computed default rank.
#[napi]
pub fn create_route_info(
    method: String,
    uri: String,
    name: Option<String>,
    rank: Option<i32>,
    format: Option<String>,
) -> RouteInfo {
    let computed_rank = rank.unwrap_or_else(|| compute_default_rank(uri.clone()));
    RouteInfo {
        name,
        method: method.to_uppercase(),
        uri,
        rank: computed_rank,
        format,
    }
}

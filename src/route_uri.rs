//! # Rocket - RouteUri NAPI Bindings
//!
//! This module implements NAPI bindings for Rocket's [`RouteUri`] struct.
//!
//! A [`RouteUri`] consists of:
//! - `base`: The mount point origin (e.g. `/` or `/base`).
//! - `unmountedOrigin`: The URI without the base mount point (e.g. `/foo/<bar>`).
//! - `origin`: The canonical URI with the base mount point (e.g. `/base/foo/<bar>`).
//! - `path`: The path portion of the canonical origin URI.
//! - `query`: The query portion of the canonical origin URI, if any.
//! - `asStr`: The full URI string representation.

use napi_derive::napi;

/// NAPI object representing a Rocket `RouteUri`.
#[napi(object)]
#[derive(Debug, Clone)]
pub struct RouteUriInfo {
    /// The mount point base path (default "/").
    pub base: String,
    /// The URI without the base mount point.
    pub unmounted_origin: String,
    /// The canonical URI including the base mount point.
    pub origin: String,
    /// The path portion of the canonical URI.
    pub path: String,
    /// The query portion of the canonical URI, if any.
    pub query: Option<String>,
    /// Full URI as string.
    pub as_str: String,
}

/// Helper function to create a `RouteUriInfo` from a base mount point and URI pattern.
#[napi]
pub fn create_route_uri(base: Option<String>, uri: String) -> RouteUriInfo {
    let base_path = base.unwrap_or_else(|| "/".to_string());
    let clean_base = if base_path == "/" {
        "/".to_string()
    } else {
        base_path.trim_end_matches('/').to_string()
    };

    let clean_uri = if !uri.starts_with('/') {
        format!("/{uri}")
    } else {
        uri
    };

    let parts: Vec<&str> = clean_uri.splitn(2, '?').collect();
    let unmounted_path = parts[0];
    let query = parts.get(1).map(|q| q.to_string());

    let canonical_path = if clean_base == "/" {
        unmounted_path.to_string()
    } else {
        format!("{}{}", clean_base, unmounted_path)
    };

    let origin = match &query {
        Some(q) if !q.is_empty() => format!("{canonical_path}?{q}"),
        _ => canonical_path.clone(),
    };

    let unmounted_origin = match &query {
        Some(q) if !q.is_empty() => format!("{unmounted_path}?{q}"),
        _ => unmounted_path.to_string(),
    };

    RouteUriInfo {
        base: clean_base,
        unmounted_origin,
        origin: origin.clone(),
        path: canonical_path,
        query,
        as_str: origin,
    }
}

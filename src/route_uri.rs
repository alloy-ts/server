//! NAPI bindings for Rocket's [`RouteUri`](rocket::route::RouteUri) struct.

use napi_derive::napi;

/// NAPI object representation of Rocket's `RouteUri`.
#[napi(object)]
pub struct RouteUri {
    pub base: String,
    pub unmounted_origin: String,
    pub origin: String,
    pub path: String,
    pub query: Option<String>,
}

/// Parses and creates a new NAPI `RouteUri` representation given base and uri strings.
#[napi]
pub fn create_route_uri(base: String, uri: String) -> RouteUri {
    let base_str = if base.is_empty() { "/".to_string() } else { base };
    let unmounted = uri.clone();
    let full_origin = if base_str == "/" {
        if uri.starts_with('/') { uri.clone() } else { format!("/{uri}") }
    } else {
        format!("{base_str}{}", if uri.starts_with('/') { uri.clone() } else { format!("/{uri}") })
    };

    let parts: Vec<&str> = full_origin.splitn(2, '?').collect();
    let path = parts[0].to_string();
    let query = parts.get(1).filter(|s| !s.is_empty()).map(|s| s.to_string());

    RouteUri {
        base: base_str,
        unmounted_origin: unmounted,
        origin: full_origin,
        path,
        query,
    }
}

//! NAPI bindings for Rocket's [`Route`] (`rocket::Route`).

use napi_derive::napi;

/// NAPI mirror of Rocket's `Route` metadata.
#[napi]
pub struct Route {
    pub name: Option<String>,
    pub method: String,
    pub uri: String,
    pub rank: i32,
    pub format: Option<String>,
}

#[napi]
impl Route {
    /// Creates a new Route specification with default rank computed by Rocket rules.
    #[napi(constructor)]
    pub fn new(method: String, uri: String, name: Option<String>) -> Self {
        let rank = compute_default_rank(&uri);
        Self {
            name,
            method: method.to_uppercase(),
            uri,
            rank,
            format: None,
        }
    }

    /// Creates a Route with a specific rank.
    #[napi]
    pub fn ranked(rank: Option<i32>, method: String, uri: String, name: Option<String>) -> Self {
        let r = rank.unwrap_or_else(|| compute_default_rank(&uri));
        Self {
            name,
            method: method.to_uppercase(),
            uri,
            rank: r,
            format: None,
        }
    }
}

fn compute_default_rank(uri: &str) -> i32 {
    let parts: Vec<&str> = uri.split('?').collect();
    let path = parts.first().copied().unwrap_or("/");
    let query = parts.get(1).copied();

    let path_color = if !path.contains('<') && !path.contains('*') {
        0
    } else if path.contains('<') && path.contains('>') {
        1
    } else {
        2
    };

    let query_color = match query {
        None => 3,
        Some(q) if !q.contains('<') => 0,
        Some(_) => 1,
    };

    match (path_color, query_color) {
        (0, 0) => -12,
        (0, 1) => -11,
        (0, 2) => -10,
        (0, 3) => -9,
        (1, 0) => -8,
        (1, 1) => -7,
        (1, 2) => -6,
        (1, 3) => -5,
        (2, 0) => -4,
        (2, 1) => -3,
        (2, 2) => -2,
        _ => -1,
    }
}

//! NAPI bindings for Rocket's `Route` struct (`rocket::Route`).

use napi_derive::napi;

#[napi]
#[derive(Clone, Debug)]
pub struct Route {
    pub name: Option<String>,
    pub method: String,
    pub uri: String,
    pub rank: i64,
    pub format: Option<String>,
}

/// Compute Rocket's default rank for a given route URI pattern.
///
/// Ranking table:
/// path    | query   | rank
/// static  | static  | -12
/// static  | partial | -11
/// static  | wild    | -10
/// static  | none    | -9
/// partial | static  | -8
/// partial | partial | -7
/// partial | wild    | -6
/// partial | none    | -5
/// wild    | static  | -4
/// wild    | partial | -3
/// wild    | wild    | -2
/// wild    | none    | -1
pub fn compute_default_rank(uri: &str) -> i64 {
    let (path_part, query_part) = match uri.split_once('?') {
        Some((p, q)) => (p, Some(q)),
        None => (uri, None),
    };

    let p_segs: Vec<&str> = path_part.split('/').filter(|s| !s.is_empty()).collect();
    let p_dyn_count = p_segs
        .iter()
        .filter(|s| s.starts_with('<') || s.starts_with(':'))
        .count();

    let path_color = if p_segs.is_empty() || p_dyn_count == 0 {
        0 // static
    } else if p_dyn_count < p_segs.len() {
        1 // partial
    } else {
        2 // wild
    };

    let query_color = match query_part {
        None => 3, // none
        Some(q) => {
            let q_segs: Vec<&str> = q.split('&').filter(|s| !s.is_empty()).collect();
            let q_dyn_count = q_segs
                .iter()
                .filter(|s| s.starts_with('<') || s.starts_with(':') || s.contains("=<") || s.contains("=:"))
                .count();

            if q_segs.is_empty() || q_dyn_count == 0 {
                0 // static
            } else if q_dyn_count < q_segs.len() {
                1 // partial
            } else {
                2 // wild
            }
        }
    };

    let rank_table = [
        [-12, -11, -10, -9], // static path
        [-8, -7, -6, -5],    // partial path
        [-4, -3, -2, -1],    // wild path
    ];

    rank_table[path_color][query_color]
}

#[napi]
impl Route {
    /// Creates a new route with default rank.
    #[napi(constructor)]
    pub fn new(method: String, uri: String, name: Option<String>, format: Option<String>) -> Self {
        let computed_rank = compute_default_rank(&uri);
        Route {
            name,
            method: method.to_uppercase(),
            uri,
            rank: computed_rank,
            format,
        }
    }

    /// Creates a new route with an explicit or default rank.
    #[napi]
    pub fn ranked(
        rank: Option<i64>,
        method: String,
        uri: String,
        name: Option<String>,
        format: Option<String>,
    ) -> Self {
        let computed_rank = rank.unwrap_or_else(|| compute_default_rank(&uri));
        Route {
            name,
            method: method.to_uppercase(),
            uri,
            rank: computed_rank,
            format,
        }
    }
}

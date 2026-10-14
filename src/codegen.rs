//! # Rocket - Code Generation NAPI Bindings

use std::collections::HashMap;
use napi_derive::napi;

#[napi(object)]
#[derive(Clone, Debug)]
pub struct RouteParam {
    pub name: String,
    pub is_trailing: bool,
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct RouteInfo {
    pub method: String,
    pub uri: String,
    pub path_params: Vec<RouteParam>,
    pub query_params: Vec<String>,
    pub rank: Option<i32>,
    pub format: Option<String>,
}

#[napi]
pub fn parse_route_pattern(pattern: String) -> RouteInfo {
    let mut path_params = Vec::new();
    let mut query_params = Vec::new();

    let (path_part, query_part) = match pattern.split_once('?') {
        Some((p, q)) => (p, Some(q)),
        None => (pattern.as_str(), None),
    };

    for seg in path_part.split('/').filter(|s| !s.is_empty()) {
        if let Some(name) = seg.strip_prefix(':') {
            path_params.push(RouteParam {
                name: name.to_string(),
                is_trailing: false,
            });
        } else if seg.starts_with('<') && seg.ends_with('>') {
            let inner = &seg[1..seg.len() - 1];
            if inner.ends_with("..") {
                path_params.push(RouteParam {
                    name: inner[..inner.len() - 2].to_string(),
                    is_trailing: true,
                });
            } else {
                path_params.push(RouteParam {
                    name: inner.to_string(),
                    is_trailing: false,
                });
            }
        }
    }

    if let Some(q) = query_part {
        for seg in q.split('&').filter(|s| !s.is_empty()) {
            if let Some(name) = seg.strip_prefix(':') {
                query_params.push(name.to_string());
            } else if seg.starts_with('<') && seg.ends_with('>') {
                let inner = &seg[1..seg.len() - 1];
                let name = inner.strip_suffix("..").unwrap_or(inner);
                query_params.push(name.to_string());
            } else if let Some((key, _)) = seg.split_once('=') {
                query_params.push(key.to_string());
            } else {
                query_params.push(seg.to_string());
            }
        }
    }

    RouteInfo {
        method: "GET".to_string(),
        uri: pattern,
        path_params,
        query_params,
        rank: None,
        format: None,
    }
}

#[napi]
pub fn build_route_uri(
    pattern: String,
    params: HashMap<String, String>,
) -> String {
    let (path_part, query_part) = match pattern.split_once('?') {
        Some((p, q)) => (p, Some(q)),
        None => (pattern.as_str(), None),
    };

    let mut new_path_segments = Vec::new();
    for seg in path_part.split('/') {
        if let Some(name) = seg.strip_prefix(':') {
            if let Some(val) = params.get(name) {
                new_path_segments.push(urlencoding_encode(val));
            } else {
                new_path_segments.push(seg.to_string());
            }
        } else if seg.starts_with('<') && seg.ends_with('>') {
            let inner = &seg[1..seg.len() - 1];
            let name = inner.strip_suffix("..").unwrap_or(inner);
            if let Some(val) = params.get(name) {
                new_path_segments.push(urlencoding_encode(val));
            } else {
                new_path_segments.push(seg.to_string());
            }
        } else {
            new_path_segments.push(seg.to_string());
        }
    }

    let mut result_path = new_path_segments.join("/");

    if let Some(q) = query_part {
        let mut query_pairs = Vec::new();
        for seg in q.split('&').filter(|s| !s.is_empty()) {
            if let Some(name) = seg.strip_prefix(':') {
                if let Some(val) = params.get(name) {
                    query_pairs.push(format!("{}={}", urlencoding_encode(name), urlencoding_encode(val)));
                }
            } else if seg.starts_with('<') && seg.ends_with('>') {
                let inner = &seg[1..seg.len() - 1];
                let name = inner.strip_suffix("..").unwrap_or(inner);
                if let Some(val) = params.get(name) {
                    query_pairs.push(format!("{}={}", urlencoding_encode(name), urlencoding_encode(val)));
                }
            } else if let Some((key, val)) = seg.split_once('=') {
                query_pairs.push(format!("{}={}", key, urlencoding_encode(val)));
            }
        }
        if !query_pairs.is_empty() {
            result_path.push('?');
            result_path.push_str(&query_pairs.join("&"));
        }
    }

    result_path
}

/// Type-safe URI builder corresponding to Rocket's `uri!` macro.
#[napi]
pub fn uri(pattern: String, params: Option<HashMap<String, String>>) -> String {
    build_route_uri(pattern, params.unwrap_or_default())
}

fn urlencoding_encode(s: &str) -> String {
    let mut encoded = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", b));
            }
        }
    }
    encoded
}

#[napi]
pub fn generate_ts_route_types(routes: Vec<RouteInfo>) -> String {
    let mut code = String::from("// Generated Rocket Route Types\n\n");
    code.push_str("export interface ApiRoutes {\n");

    for route in routes {
        code.push_str(&format!("  \"{} {}\": {{\n", route.method, route.uri));
        code.push_str("    params: {\n");
        for param in route.path_params {
            code.push_str(&format!("      {}: string;\n", param.name));
        }
        code.push_str("    };\n");
        code.push_str("    query: {\n");
        for q in route.query_params {
            code.push_str(&format!("      {}?: string;\n", q));
        }
        code.push_str("    };\n");
        code.push_str("  };\n");
    }

    code.push_str("}\n");
    code
}

/// Codegen helper: generate TypeScript definition interfaces for registered route patterns.
#[napi]
pub fn generate_routes_type_definitions(patterns: Vec<String>) -> String {
    let routes = patterns.into_iter().map(parse_route_pattern).collect();
    generate_ts_route_types(routes)
}

//! # Rocket - Code Generation (NAPI Bindings)
//!
//! Exposes Rocket's code generation facilities to Node.js / TypeScript:
//! route parsing, pattern translation, type-safe URI building, form field
//! validation, and route table metadata manifest generation.

use std::collections::HashMap;
use napi_derive::napi;

/// Parsed route pattern metadata.
#[napi(object)]
pub struct RoutePatternInfo {
    pub raw_pattern: String,
    pub rocket_pattern: String,
    pub path_params: Vec<String>,
    pub is_wildcard: bool,
}

/// Parse a TS/Alloy route pattern (`/users/:id`, `/api/*`) into Rocket pattern syntax
/// and extract path parameter names.
#[napi]
pub fn parse_route_pattern(pattern: String) -> RoutePatternInfo {
    let mut rocket_pattern = String::with_capacity(pattern.len());
    let mut path_params = Vec::new();
    let mut chars = pattern.chars().peekable();
    let is_wildcard = pattern.contains('*');

    while let Some(c) = chars.next() {
        if c == ':' {
            let mut name = String::new();
            while let Some(&nc) = chars.peek() {
                if nc.is_alphanumeric() || nc == '_' {
                    name.push(nc);
                    chars.next();
                } else {
                    break;
                }
            }
            if name.is_empty() {
                rocket_pattern.push(':');
            } else {
                path_params.push(name.clone());
                rocket_pattern.push_str(&format!("<{name}>"));
            }
        } else if c == '*' {
            rocket_pattern.push_str("<_..>");
        } else {
            rocket_pattern.push(c);
        }
    }

    RoutePatternInfo {
        raw_pattern: pattern,
        rocket_pattern,
        path_params,
        is_wildcard,
    }
}

/// NAPI equivalent of Rocket's `uri!` macro: interpolates parameters into
/// a route pattern to construct a URL-safe URI.
#[napi]
pub fn build_route_uri(template: String, params: Option<HashMap<String, String>>) -> String {
    let info = parse_route_pattern(template);
    let mut uri = info.raw_pattern;

    if let Some(params_map) = params {
        for (key, value) in params_map {
            let placeholder_colon = format!(":{key}");
            let placeholder_angle = format!("<{key}>");
            let encoded_value = urlencoding_encode(&value);

            if uri.contains(&placeholder_colon) {
                uri = uri.replace(&placeholder_colon, &encoded_value);
            } else if uri.contains(&placeholder_angle) {
                uri = uri.replace(&placeholder_angle, &encoded_value);
            }
        }
    }

    uri
}

/// Helper for `#[catch]` status parsing: turns `"404"`, `"default"`, or numeric status
/// into Rocket catcher code representation.
#[napi]
pub fn parse_catcher_code(code_str: String) -> Option<u32> {
    let s = code_str.trim().to_lowercase();
    if s == "default" || s == "_" {
        None
    } else {
        s.parse::<u32>().ok()
    }
}

/// Generate route table manifest metadata for dynamic TS route table inspection.
#[napi]
pub fn generate_route_manifest(routes: Vec<String>) -> Vec<RoutePatternInfo> {
    routes.into_iter().map(parse_route_pattern).collect()
}

/// Basic URL encoding helper.
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

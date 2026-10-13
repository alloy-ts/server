//! # Rocket - Code Generation for TypeScript / Node.js
//!
//! Rocket's Rust procedural macros (`#[get]`, `routes!`, `#[derive(FromForm)]`, `uri!`)
//! compile at Rust build-time. This module provides NAPI bindings and helpers that
//! bring equivalent code-generation, route metadata parsing, and URI building
//! capabilities to TypeScript and Node.js.

use std::collections::HashMap;
use napi_derive::napi;

/// Metadata describing a generated Rocket route.
#[napi(object)]
pub struct RouteMetadata {
    pub method: String,
    pub uri: String,
    pub name: Option<String>,
    pub rank: Option<i32>,
    pub format: Option<String>,
}

/// Metadata describing a Rocket catcher.
#[napi(object)]
pub struct CatcherMetadata {
    pub code: Option<u32>,
    pub name: Option<String>,
}

/// Parse and build route metadata for codegen, matching Rocket's `#[route]` attribute syntax.
#[napi]
pub fn parse_route_attribute(
    method: String,
    uri: String,
    name: Option<String>,
    rank: Option<i32>,
    format: Option<String>,
) -> RouteMetadata {
    RouteMetadata {
        method: method.to_uppercase(),
        uri,
        name,
        rank,
        format,
    }
}

/// Type-safe URI builder corresponding to Rocket's `uri!` macro.
/// Interpolates named path parameters into the URI pattern and URL-encodes dynamic values.
#[napi]
pub fn build_uri(uri_template: String, params: HashMap<String, String>) -> String {
    let mut result = uri_template.clone();
    for (key, val) in params {
        let encoded = urlencoding_encode(&val);
        let target_bracket = format!("<{}>", key);
        let target_colon = format!(":{}", key);
        result = result.replace(&target_bracket, &encoded);
        result = result.replace(&target_colon, &encoded);
    }
    result
}

/// Simple percent-encoding helper for URI query and path parameters.
fn urlencoding_encode(s: &str) -> String {
    let mut encoded = String::new();
    for b in s.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", b));
            }
        }
    }
    encoded
}

/// Parse a catcher attribute declaration (e.g., `404` or `"default"`).
#[napi]
pub fn parse_catcher_attribute(status_or_default: String, name: Option<String>) -> CatcherMetadata {
    let code = status_or_default.parse::<u32>().ok();
    CatcherMetadata { code, name }
}

/// Generates TypeScript route mapping code given a list of route metadata.
#[napi]
pub fn generate_ts_route_manifest(routes: Vec<RouteMetadata>) -> String {
    let mut code = String::from("// Auto-generated Rocket Route Manifest\nexport const routes = {\n");
    for r in routes {
        let route_name = r.name.unwrap_or_else(|| "handler".to_string());
        code.push_str(&format!(
            "  \"{}\": {{ method: \"{}\", name: \"{}\" }},\n",
            r.uri, r.method, route_name
        ));
    }
    code.push_str("};\n");
    code
}

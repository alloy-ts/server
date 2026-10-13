//! # Rocket - Code Generation for TypeScript & NAPI
//!
//! This module implements the code generation support for Rocket in Node.js /
//! TypeScript environments.
//!
//! In Rust, Rocket's code generation (`#[get]`, `#[post]`, `routes!`, `uri!`,
//! `derive(FromForm)`) relies on procedural macros evaluated at compile-time.
//! When bridging Rocket to Node.js and TypeScript via NAPI:
//!
//! 1. **Runtime Binding Layer**: The C-ABI / NAPI layer (`server.rs`, `rocket.rs`)
//!    bridges Node.js route handlers into Rocket's multi-threaded Tokio runtime.
//! 2. **TypeScript Decorators & DSL**: TypeScript code generators and decorators
//!    (e.g., `@get("/path")`, `@post("/path")`, `@catch(404)`) capture route
//!    metadata and register routes dynamically or at bundle-time.
//! 3. **Type-Safe Codegen Helpers**: This module (`codegen.rs`) provides NAPI
//!    helpers to parse route URIs, extract dynamic parameter metadata, translate
//!    route patterns, and emit TypeScript client/server code for type-safe `uri!`
//!    builders and OpenAPI schemas.

use std::collections::HashMap;
use napi_derive::napi;

/// Metadata for a single route parameter extracted during codegen analysis.
#[napi(object)]
pub struct RouteParamInfo {
    pub name: String,
    pub is_trailing: bool,
    pub is_query: bool,
}

/// Metadata for a route, parsed for code generation and TypeScript type generation.
#[napi(object)]
pub struct RouteMetadata {
    pub method: Option<String>,
    pub path: String,
    pub rocket_path: String,
    pub params: Vec<RouteParamInfo>,
}

/// Input structure for bulk route codegen operations.
#[napi(object)]
pub struct RouteMetadataInput {
    pub name: String,
    pub method: String,
    pub path: String,
}

/// Parses a route URI pattern and extracts parameter metadata for code generation.
///
/// Supports both Bun/Express parameter syntax (`/user/:id/*`) and Rocket parameter
/// syntax (`/user/<id>/<_..>`).
#[napi]
pub fn parse_route_uri(path: String, method: Option<String>) -> RouteMetadata {
    let rocket_path = translate_pattern(path.clone());
    let params = extract_route_params(&rocket_path);

    RouteMetadata {
        method,
        path,
        rocket_path,
        params,
    }
}

/// Translates Express/Bun style route patterns (`:param`, `*`) into Rocket syntax
/// (`<param>`, `<_..>`).
#[napi]
pub fn translate_pattern(pattern: String) -> String {
    let mut out = String::with_capacity(pattern.len());
    let mut chars = pattern.chars().peekable();
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
                out.push(':');
            } else {
                out.push_str(&format!("<{name}>"));
            }
        } else if c == '*' {
            out.push_str("<_..>");
        } else {
            out.push(c);
        }
    }
    out
}

/// Align a Rocket route pattern's `<param>` segments with actual path segments.
pub fn extract_params(pattern: &str, path: &str) -> HashMap<String, String> {
    let p_segs: Vec<&str> = pattern
        .trim_start_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();
    let a_segs: Vec<&str> = path
        .trim_start_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();
    let mut params = HashMap::new();
    for (idx, seg) in p_segs.iter().enumerate() {
        if let Some(inner) = seg.strip_prefix('<') {
            let name = inner
                .strip_suffix("..>")
                .or_else(|| inner.strip_suffix('>'));
            if let Some(name) = name {
                if name != "_" {
                    if let Some(v) = a_segs.get(idx) {
                        params.insert(name.to_string(), (*v).to_string());
                    }
                }
            }
        }
    }
    params
}

/// Helper function to extract parameter info from Rocket route patterns.
fn extract_route_params(pattern: &str) -> Vec<RouteParamInfo> {
    let mut params = Vec::new();
    let parts: Vec<&str> = pattern.split('?').collect();
    let path_part = parts[0];
    let query_part = parts.get(1).copied().unwrap_or("");

    // Extract path parameters
    for seg in path_part.split('/') {
        if let Some(inner) = seg.strip_prefix('<') {
            let is_trailing = inner.ends_with("..>");
            let name = inner
                .strip_suffix("..>")
                .or_else(|| inner.strip_suffix('>'))
                .unwrap_or("")
                .to_string();
            if !name.is_empty() && name != "_" {
                params.push(RouteParamInfo {
                    name,
                    is_trailing,
                    is_query: false,
                });
            }
        }
    }

    // Extract query parameters
    if !query_part.is_empty() {
        for seg in query_part.split('&') {
            if let Some(inner) = seg.strip_prefix('<') {
                let is_trailing = inner.ends_with("..>");
                let name = inner
                    .strip_suffix("..>")
                    .or_else(|| inner.strip_suffix('>'))
                    .unwrap_or("")
                    .to_string();
                if !name.is_empty() && name != "_" {
                    params.push(RouteParamInfo {
                        name,
                        is_trailing,
                        is_query: true,
                    });
                }
            }
        }
    }

    params
}

/// Generates TypeScript code string for type-safe route URI builders (`uri!`).
///
/// Converts a list of route metadata into typed TypeScript function helpers.
#[napi]
pub fn generate_ts_uri_builder(routes: Vec<RouteMetadataInput>) -> String {
    let mut code = String::new();
    code.push_str("// Auto-generated by Rocket NAPI Codegen\n");
    code.push_str("export const uri = {\n");

    for route in routes {
        let meta = parse_route_uri(route.path.clone(), Some(route.method.clone()));
        let mut fn_args = Vec::new();
        for param in &meta.params {
            fn_args.push(format!("{}: string | number", param.name));
        }
        let args_str = fn_args.join(", ");

        let mut path_expr = meta.path.clone();
        for param in &meta.params {
            path_expr = path_expr.replace(&format!(":{}", param.name), &format!("${{{}}}", param.name));
            path_expr = path_expr.replace(&format!("<{}>", param.name), &format!("${{{}}}", param.name));
        }

        code.push_str(&format!(
            "  {}: ({}) => `{}`,\n",
            route.name, args_str, path_expr
        ));
    }

    code.push_str("};\n");
    code
}

/// Parses a catcher HTTP status code string (`"404"`, `"500"`, `"default"`).
/// Returns `Some(code)` for integer status, or `None` for `"default"`.
#[napi]
pub fn parse_catcher_status(status_str: String) -> Option<u32> {
    if status_str.eq_ignore_ascii_case("default") {
        None
    } else {
        status_str.parse::<u32>().ok()
    }
}

//! # Rocket - Code Generation NAPI Bindings
//!
//! This module provides NAPI bindings for Rocket's code generation facilities.
//! In Rust, Rocket uses procedural macros (`#[get]`, `routes!`, `uri!`, `derive(FromForm)`)
//! defined in `rocket_codegen`. Because proc macros cannot cross the N-API boundary into
//! Node.js/TypeScript at runtime, this module implements the NAPI equivalents of those
//! responsibilities:
//!
//! 1. Route attribute parsing (`parse_route_attribute`): validates and extracts route
//!    metadata according to Rocket's route attribute grammar.
//! 2. Catch attribute parsing (`parse_catch_attribute`): handles `#[catch(status)]` and
//!    `#[catch(default)]`.
//! 3. Type-safe URI generation (`generate_uri`): formats URL-safe URIs following Rocket's
//!    `uri!` macro semantics (URL percent-encoding, path and query parameter interpolation).
//! 4. TypeScript route manifest generation (`generate_ts_route_manifest`): generates
//!    TypeScript route definitions and type-safe URI helpers.

use std::collections::HashMap;
use napi::bindgen_prelude::Status;
use napi_derive::napi;

/// Parsed route metadata mirroring the items emitted by Rocket's `route` macro attribute.
#[napi(object)]
pub struct ParsedRoute {
    pub method: Option<String>,
    pub uri: String,
    pub path: String,
    pub query: Option<String>,
    pub path_params: Vec<String>,
    pub query_params: Vec<String>,
    pub rank: Option<i32>,
    pub format: Option<String>,
    pub data_param: Option<String>,
}

/// Parsed catcher metadata mirroring Rocket's `catch` macro attribute.
#[napi(object)]
pub struct ParsedCatch {
    pub code: Option<u32>,
    pub is_default: bool,
}

/// Metadata input for generating TypeScript route definitions.
#[napi(object)]
pub struct RouteMetadataInput {
    pub name: String,
    pub method: String,
    pub uri: String,
}

/// Parse a Rocket route attribute specification (e.g., `"/person/<name>?<age>", rank = 1, format = "json"`).
///
/// Follows Rocket's attribute grammar:
/// ```text
/// route := '"' uri ('?' query)? '"' (',' parameter)*
/// parameter := 'rank' '=' INTEGER | 'format' '=' '"' MEDIA_TYPE '"' | 'data' '=' '"' SINGLE_PARAM '"'
/// ```
#[napi]
pub fn parse_route_attribute(uri_spec: String, args: Option<String>) -> napi::Result<ParsedRoute> {
    let full_spec = match args {
        Some(a) if !a.trim().is_empty() => format!("{}, {}", uri_spec, a),
        _ => uri_spec.clone(),
    };

    let mut parts = full_spec.split(',');
    let raw_uri = parts
        .next()
        .ok_or_else(|| napi::Error::new(Status::InvalidArg, "empty route URI"))?
        .trim()
        .trim_matches('"')
        .to_string();

    let mut rank = None;
    let mut format_type = None;
    let mut data_param = None;

    for param in parts {
        let param = param.trim();
        if let Some((key, val)) = param.split_once('=') {
            let key = key.trim();
            let val = val.trim().trim_matches('"');
            match key {
                "rank" => {
                    rank = val.parse::<i32>().ok();
                }
                "format" => {
                    format_type = Some(val.to_string());
                }
                "data" => {
                    let cleaned = val.trim_start_matches('<').trim_end_matches('>');
                    data_param = Some(cleaned.to_string());
                }
                _ => {}
            }
        }
    }

    let (path, query) = match raw_uri.split_once('?') {
        Some((p, q)) => (p.to_string(), Some(q.to_string())),
        None => (raw_uri.clone(), None),
    };

    let path_params = extract_segment_params(&path);
    let query_params = query
        .as_ref()
        .map(|q| extract_segment_params(q))
        .unwrap_or_default();

    Ok(ParsedRoute {
        method: None,
        uri: raw_uri,
        path,
        query,
        path_params,
        query_params,
        rank,
        format: format_type,
        data_param,
    })
}

/// Helper to extract parameter identifiers enclosed in `<ident>` or `<ident..>`.
fn extract_segment_params(segment: &str) -> Vec<String> {
    let mut params = Vec::new();
    let mut chars = segment.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '<' {
            let mut name = String::new();
            while let Some(&c) = chars.peek() {
                if c == '>' {
                    chars.next();
                    break;
                }
                name.push(chars.next().unwrap());
            }
            let cleaned = name.trim_end_matches("..");
            if !cleaned.is_empty() && cleaned != "_" {
                params.push(cleaned.to_string());
            }
        }
    }
    params
}

/// Parse a Rocket catch attribute specification (`"404"` or `"default"`).
#[napi]
pub fn parse_catch_attribute(args: String) -> napi::Result<ParsedCatch> {
    let trimmed = args.trim().trim_matches('"');
    if trimmed.eq_ignore_ascii_case("default") {
        Ok(ParsedCatch {
            code: None,
            is_default: true,
        })
    } else {
        let code = trimmed
            .parse::<u32>()
            .map_err(|_| napi::Error::new(Status::InvalidArg, format!("invalid status code: {args}")))?;
        Ok(ParsedCatch {
            code: Some(code),
            is_default: false,
        })
    }
}

/// URL-encode a URI path/query segment (RFC 3986 percent-encoding).
fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push_str(&format!("%{:02X}", b));
            }
        }
    }
    out
}

/// Type-safe URI builder matching Rocket's `uri!` macro semantics.
///
/// Accepts a route URI template (e.g. `"/person/<name>?<age>"`) and a map of parameter values.
/// Interpolates path parameters with URL percent-encoding and appends formatted query parameters.
#[napi]
pub fn generate_uri(
    template: String,
    params: Option<HashMap<String, String>>,
) -> napi::Result<String> {
    let params = params.unwrap_or_default();
    let (path_template, query_template) = match template.split_once('?') {
        Some((p, q)) => (p, Some(q)),
        None => (template.as_str(), None),
    };

    let mut result_path = String::new();
    let path_segments = path_template.split('/');
    for (i, seg) in path_segments.enumerate() {
        if i > 0 {
            result_path.push('/');
        }
        if seg.starts_with('<') && seg.ends_with('>') {
            let inner = &seg[1..seg.len() - 1];
            let name = inner.trim_end_matches("..");
            if let Some(val) = params.get(name) {
                result_path.push_str(&percent_encode(val));
            } else {
                result_path.push_str(seg);
            }
        } else {
            result_path.push_str(seg);
        }
    }

    let mut query_pairs = Vec::new();
    if let Some(query_str) = query_template {
        for pair in query_str.split('&') {
            if pair.is_empty() {
                continue;
            }
            let (raw_key, val_template) = match pair.split_once('=') {
                Some((k, v)) => (k, v),
                None => (pair, pair),
            };

            let clean_key = if raw_key.starts_with('<') && raw_key.ends_with('>') {
                let inner = &raw_key[1..raw_key.len() - 1];
                inner.trim_end_matches("..")
            } else {
                raw_key
            };

            let param_name = if val_template.starts_with('<') && val_template.ends_with('>') {
                let inner = &val_template[1..val_template.len() - 1];
                inner.trim_end_matches("..")
            } else {
                clean_key
            };

            if let Some(val) = params.get(param_name) {
                if val != "_" && !val.is_empty() {
                    query_pairs.push(format!("{}={}", percent_encode(clean_key), percent_encode(val)));
                }
            }
        }
    }

    if !query_pairs.is_empty() {
        result_path.push('?');
        result_path.push_str(&query_pairs.join("&"));
    }

    Ok(result_path)
}

/// Generates a TypeScript module manifest containing route definitions and type-safe URI helpers.
#[napi]
pub fn generate_ts_route_manifest(routes: Vec<RouteMetadataInput>) -> String {
    let mut ts = String::new();
    ts.push_str("// Auto-generated Rocket route manifest\n");
    ts.push_str("import { generateUri } from \"@alloy-ts/server\";\n\n");
    ts.push_str("export const routes = [\n");
    for r in &routes {
        ts.push_str(&format!(
            "  {{ name: {:?}, method: {:?}, uri: {:?} }},\n",
            r.name, r.method, r.uri
        ));
    }
    ts.push_str("];\n\n");

    ts.push_str("export const uri = {\n");
    for r in &routes {
        let parsed = parse_route_attribute(r.uri.clone(), None).unwrap_or(ParsedRoute {
            method: None,
            uri: r.uri.clone(),
            path: r.uri.clone(),
            query: None,
            path_params: vec![],
            query_params: vec![],
            rank: None,
            format: None,
            data_param: None,
        });

        let mut param_types = Vec::new();
        for p in &parsed.path_params {
            param_types.push(format!("{}: string", p));
        }
        for q in &parsed.query_params {
            param_types.push(format!("{}?: string", q));
        }

        let params_sig = if param_types.is_empty() {
            "".to_string()
        } else {
            format!("params: {{ {} }}", param_types.join("; "))
        };

        let param_arg = if param_types.is_empty() {
            "undefined"
        } else {
            "params as unknown as Record<string, string>"
        };

        ts.push_str(&format!(
            "  {}({}): string {{\n",
            r.name, params_sig
        ));
        ts.push_str(&format!(
            "    return generateUri({:?}, {});\n",
            r.uri, param_arg
        ));
        ts.push_str("  },\n");
    }
    ts.push_str("};\n");

    ts
}

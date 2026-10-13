//! # Rocket - Code Generation NAPI Bindings
//!
//! This module implements the code generation facilities of Rocket adapted for
//! TypeScript / Node.js via NAPI.
//!
//! In Rust, Rocket's `#[get]`, `routes!`, `derive(FromForm)`, `uri!`, etc. are
//! procedural macros. For TypeScript, this module exposes the equivalent parser,
//! metadata generator, URI builder, and route registry helper so that TS code
//! generation tools, decorators (`@Get`, `@Post`), and type-safe URI builders
//! can interact directly with Rocket's codegen semantics.

use std::collections::HashMap;
use napi::bindgen_prelude::*;
use napi_derive::napi;

/// Parameter metadata extracted from Rocket route paths (e.g. `<foo>`, `<baz..>`).
#[derive(Clone, Debug)]
#[napi(object)]
pub struct ParamMeta {
    pub name: String,
    pub is_trailing: bool,
    pub is_query: bool,
}

/// Metadata generated from parsing a Rocket route attribute e.g.
/// `#[get("/<foo>/bar/<baz..>?<msg>", data = "<form>", rank = 1, format = "json")]`.
#[derive(Clone, Debug)]
#[napi(object)]
pub struct RouteMeta {
    pub method: String,
    pub uri: String,
    pub path: String,
    pub query: Option<String>,
    pub rank: Option<i32>,
    pub format: Option<String>,
    pub data_param: Option<String>,
    pub path_params: Vec<ParamMeta>,
    pub query_params: Vec<ParamMeta>,
}

/// Metadata generated from parsing a Rocket catcher attribute e.g. `#[catch(404)]` or `#[catch(default)]`.
#[derive(Clone, Debug)]
#[napi(object)]
pub struct CatcherMeta {
    pub code: Option<u32>,
    pub is_default: bool,
}

/// Field metadata for `FromForm` form parsing and validation codegen.
#[derive(Clone, Debug)]
#[napi(object)]
pub struct FormFieldMeta {
    pub name: String,
    pub mapped_name: Option<String>,
    pub default_value: Option<String>,
    pub validate_rule: Option<String>,
}

/// Parse a Rocket route attribute and path into `RouteMeta`.
///
/// Example inputs:
/// - `method = "GET"`, `attr = "/person/<name>?<age>&<rest..>"`
/// - `method = "POST"`, `attr = "\"/api/posts\", data = \"<form>\", rank = 1, format = \"json\""`
#[napi]
pub fn parse_route_attribute(method: Option<String>, attr: String) -> napi::Result<RouteMeta> {
    let raw = attr.trim();
    let method = method.unwrap_or_else(|| "GET".to_string()).to_uppercase();

    let mut route_uri = String::new();
    let mut rank: Option<i32> = None;
    let mut format: Option<String> = None;
    let mut data_param: Option<String> = None;

    // Check if attribute contains key-value options like `data = "<form>", rank = 1`
    if raw.starts_with('"') || raw.starts_with('/') {
        // Simple case: uri string or comma separated
        let parts: Vec<&str> = raw.split(',').map(|s| s.trim()).collect();
        if !parts.is_empty() {
            route_uri = parts[0].trim_matches('"').to_string();
        }

        for part in parts.iter().skip(1) {
            if let Some((k, v)) = part.split_once('=') {
                let k = k.trim();
                let v = v.trim().trim_matches('"');
                match k {
                    "rank" => rank = v.parse::<i32>().ok(),
                    "format" => format = Some(v.to_string()),
                    "data" => {
                        let cleaned = v.trim_matches('<').trim_matches('>').to_string();
                        data_param = Some(cleaned);
                    }
                    _ => {}
                }
            }
        }
    } else {
        route_uri = raw.to_string();
    }

    if route_uri.is_empty() {
        route_uri = "/".to_string();
    }

    let (path, query) = match route_uri.split_once('?') {
        Some((p, q)) => (p.to_string(), Some(q.to_string())),
        None => (route_uri.clone(), None),
    };

    let mut path_params = Vec::new();
    for seg in path.split('/') {
        if let Some(param) = parse_param_segment(seg, false) {
            path_params.push(param);
        }
    }

    let mut query_params = Vec::new();
    if let Some(q) = &query {
        for seg in q.split('&') {
            if let Some(param) = parse_param_segment(seg, true) {
                query_params.push(param);
            }
        }
    }

    Ok(RouteMeta {
        method,
        uri: route_uri,
        path,
        query,
        rank,
        format,
        data_param,
        path_params,
        query_params,
    })
}

fn parse_param_segment(seg: &str, is_query: bool) -> Option<ParamMeta> {
    let seg = seg.trim();
    if let Some(inner) = seg.strip_prefix('<') {
        let is_trailing = inner.ends_with("..>");
        let name = inner
            .strip_suffix("..>")
            .or_else(|| inner.strip_suffix('>'))
            .unwrap_or(inner);

        if !name.is_empty() && name != "_" {
            return Some(ParamMeta {
                name: name.to_string(),
                is_trailing,
                is_query,
            });
        }
    }
    None
}

/// Parse a Rocket catcher attribute e.g. `404` or `default`.
#[napi]
pub fn parse_catch_attribute(attr: String) -> napi::Result<CatcherMeta> {
    let trimmed = attr.trim().trim_matches('"').trim_matches('\'');
    if trimmed.eq_ignore_ascii_case("default") {
        Ok(CatcherMeta {
            code: None,
            is_default: true,
        })
    } else if let Ok(code) = trimmed.parse::<u32>() {
        Ok(CatcherMeta {
            code: Some(code),
            is_default: false,
        })
    } else {
        Err(Error::new(
            Status::InvalidArg,
            format!("invalid catch attribute: `{attr}`"),
        ))
    }
}

/// Build a URL-safe, type-safe route URI given a pattern and parameter map.
///
/// Mirrors Rocket's `uri!` macro.
/// Pattern example: `/person/<name>?<age>&<rest..>`
#[napi]
pub fn build_uri(
    pattern: String,
    params: HashMap<String, String>,
    prefix: Option<String>,
    suffix: Option<String>,
) -> napi::Result<String> {
    let meta = parse_route_attribute(None, pattern)?;
    let mut built_path = meta.path;

    for param in meta.path_params {
        let val = params.get(&param.name).ok_or_else(|| {
            Error::new(
                Status::InvalidArg,
                format!("missing required path parameter `{}`", param.name),
            )
        })?;
        let encoded = urlencoding_encode(val);
        let target = if param.is_trailing {
            format!("<{}..>", param.name)
        } else {
            format!("<{}>", param.name)
        };
        built_path = built_path.replace(&target, &encoded);
    }

    let mut query_parts = Vec::new();
    for param in meta.query_params {
        if let Some(val) = params.get(&param.name) {
            let encoded = urlencoding_encode(val);
            query_parts.push(format!("{}={}", param.name, encoded));
        }
    }

    let mut full_uri = built_path;
    if !query_parts.is_empty() {
        full_uri.push('?');
        full_uri.push_str(&query_parts.join("&"));
    }

    if let Some(pre) = prefix {
        let pre_clean = pre.trim_end_matches('/');
        if !full_uri.starts_with('/') {
            full_uri = format!("{}/{}", pre_clean, full_uri);
        } else {
            full_uri = format!("{}{}", pre_clean, full_uri);
        }
    }

    if let Some(suf) = suffix {
        full_uri.push_str(&suf);
    }

    Ok(full_uri)
}

fn urlencoding_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push_str(&format!("%{:02X}", b));
            }
        }
    }
    out
}

/// Generate TypeScript route mapping code or type definitions from a set of `RouteMeta`.
#[napi]
pub fn generate_ts_routes(routes: Vec<RouteMeta>) -> String {
    let mut out = String::new();
    out.push_str("// Auto-generated by Rocket Codegen NAPI\n");
    out.push_str("export const routes = [\n");

    for r in routes {
        out.push_str(&format!(
            "  {{ method: {:?}, uri: {:?}, path: {:?}, rank: {:?}, format: {:?} }},\n",
            r.method, r.uri, r.path, r.rank, r.format
        ));
    }

    out.push_str("];\n");
    out
}

/// Generate form validation function for `FromForm` derived types.
#[napi]
pub fn generate_form_schema(fields: Vec<FormFieldMeta>) -> String {
    let mut out = String::new();
    out.push_str("export function validateForm(data: Record<string, any>) {\n");
    out.push_str("  const errors: Record<string, string> = {};\n");

    for field in fields {
        let key = field.mapped_name.as_ref().unwrap_or(&field.name);
        if let Some(rule) = field.validate_rule {
            out.push_str(&format!(
                "  if (!data[{:?}]) {{ errors[{:?}] = \"Rule failed: {}\"; }}\n",
                key, key, rule
            ));
        }
    }

    out.push_str("  return { valid: Object.keys(errors).length === 0, errors };\n");
    out.push_str("}\n");
    out
}

/// Main Codegen NAPI wrapper class.
#[napi]
pub struct Codegen;

#[napi]
impl Codegen {
    /// Parse route attribute
    #[napi]
    pub fn parse_route(method: Option<String>, attr: String) -> napi::Result<RouteMeta> {
        parse_route_attribute(method, attr)
    }

    /// Parse catch attribute
    #[napi]
    pub fn parse_catch(attr: String) -> napi::Result<CatcherMeta> {
        parse_catch_attribute(attr)
    }

    /// Build URI
    #[napi]
    pub fn build_uri(
        pattern: String,
        params: HashMap<String, String>,
        prefix: Option<String>,
        suffix: Option<String>,
    ) -> napi::Result<String> {
        build_uri(pattern, params, prefix, suffix)
    }

    /// Generate TS routes list
    #[napi]
    pub fn generate_ts_routes(routes: Vec<RouteMeta>) -> String {
        generate_ts_routes(routes)
    }

    /// Generate Form Schema
    #[napi]
    pub fn generate_form_schema(fields: Vec<FormFieldMeta>) -> String {
        generate_form_schema(fields)
    }
}

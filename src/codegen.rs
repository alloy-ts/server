// # Rocket - Code Generation
//
// This module implements the code generation portions of Rocket for NAPI / TypeScript.
// This includes route generation, attribute metadata parsing, URI generation,
// catcher generation, and TypeScript declaration code generation.

use std::collections::HashMap;
use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::server::translate_pattern;

static ROCKET_IDENT_PREFIX: &str = "__rocket_";

// ---------------------------------------------------------------------------
// Codegen NAPI Types
// ---------------------------------------------------------------------------

#[napi(object)]
pub struct CodegenParam {
    pub name: String,
    pub kind: String, // "path_single", "path_multi", "query", "data", "guard"
    pub ts_type: String,
}

#[napi(object)]
pub struct CodegenRouteSpec {
    pub method: String,
    pub uri: String,
    pub rank: Option<i32>,
    pub format: Option<String>,
    pub data: Option<String>,
    pub name: Option<String>,
}

#[napi(object)]
pub struct CodegenRoute {
    pub name: String,
    pub method: String,
    pub path: String,
    pub rocket_path: String,
    pub rank: Option<i32>,
    pub format: Option<String>,
    pub data: Option<String>,
    pub params: Vec<CodegenParam>,
    pub macro_ident: String,
}

#[napi(object)]
pub struct CodegenCatcher {
    pub name: String,
    pub code: Option<u16>,
    pub is_default: bool,
}

#[napi(object)]
pub struct CodegenUri {
    pub uri: String,
    pub path: String,
    pub query: Option<String>,
    pub params: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn debug_log(msg: &str) {
    if std::env::var_os("ROCKET_CODEGEN_DEBUG").is_some() {
        println!("[ROCKET CODEGEN DEBUG] {msg}");
    }
}

fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
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

/// Parse route path parameters (`<ident>`, `<ident..>`, `:ident`, `*`).
fn parse_route_params(path: &str, data: &Option<String>) -> Vec<CodegenParam> {
    let mut params = Vec::new();
    let segments = path.split('/').filter(|s| !s.is_empty());

    for seg in segments {
        if seg.starts_with(':') {
            let name = seg[1..].to_string();
            if !name.is_empty() {
                params.push(CodegenParam {
                    name,
                    kind: "path_single".to_string(),
                    ts_type: "string".to_string(),
                });
            }
        } else if seg == "*" {
            params.push(CodegenParam {
                name: "wildcard".to_string(),
                kind: "path_multi".to_string(),
                ts_type: "string[]".to_string(),
            });
        } else if seg.starts_with('<') && seg.ends_with('>') {
            let inner = &seg[1..seg.len() - 1];
            if inner.ends_with("..") {
                let name = inner[..inner.len() - 2].to_string();
                params.push(CodegenParam {
                    name,
                    kind: "path_multi".to_string(),
                    ts_type: "string[]".to_string(),
                });
            } else if inner != "_" {
                params.push(CodegenParam {
                    name: inner.to_string(),
                    kind: "path_single".to_string(),
                    ts_type: "string".to_string(),
                });
            }
        }
    }

    if let Some((_, query)) = path.split_once('?') {
        for qseg in query.split('&') {
            if qseg.starts_with('<') && qseg.ends_with('>') {
                let name = qseg[1..qseg.len() - 1].trim_end_matches("..").to_string();
                if name != "_" {
                    params.push(CodegenParam {
                        name,
                        kind: "query".to_string(),
                        ts_type: "string | undefined".to_string(),
                    });
                }
            } else if !qseg.is_empty() {
                let name = qseg.split('=').next().unwrap_or(qseg).to_string();
                params.push(CodegenParam {
                    name,
                    kind: "query".to_string(),
                    ts_type: "string | undefined".to_string(),
                });
            }
        }
    }

    if let Some(d) = data {
        let name = d.trim_matches('<').trim_matches('>').to_string();
        params.push(CodegenParam {
            name,
            kind: "data".to_string(),
            ts_type: "unknown".to_string(),
        });
    }

    params
}

// ---------------------------------------------------------------------------
// Public Codegen NAPI Functions
// ---------------------------------------------------------------------------

/// Generate metadata for a single Rocket route spec.
#[napi]
pub fn generate_route(spec: CodegenRouteSpec) -> Result<CodegenRoute> {
    let method = spec.method.to_uppercase();
    let name = spec
        .name
        .unwrap_or_else(|| format!("{}_{}", method.to_lowercase(), spec.uri.replace('/', "_")));
    let rocket_path = translate_pattern(&spec.uri);
    let params = parse_route_params(&spec.uri, &spec.data);
    let macro_ident = format!("{ROCKET_IDENT_PREFIX}route_{name}");

    let route = CodegenRoute {
        name,
        method,
        path: spec.uri,
        rocket_path,
        rank: spec.rank,
        format: spec.format,
        data: spec.data,
        params,
        macro_ident,
    };

    debug_log(&format!("Generated route: {} {}", route.method, route.path));
    Ok(route)
}

/// Batch generate Rocket routes.
#[napi]
pub fn generate_routes(specs: Vec<CodegenRouteSpec>) -> Result<Vec<CodegenRoute>> {
    specs.into_iter().map(generate_route).collect()
}

/// Generate catcher metadata.
#[napi]
pub fn generate_catcher(code: Option<u16>, name: Option<String>) -> Result<CodegenCatcher> {
    let is_default = code.is_none();
    let catcher_name = name.unwrap_or_else(|| match code {
        Some(c) => format!("catcher_{c}"),
        None => "catcher_default".to_string(),
    });

    let catcher = CodegenCatcher {
        name: catcher_name,
        code,
        is_default,
    };

    debug_log(&format!("Generated catcher: {:?}", catcher.code));
    Ok(catcher)
}

/// Type-safe URI builder matching Rocket's `uri!` macro dynamic semantics.
#[napi]
pub fn generate_uri(
    route_path: String,
    params: Option<HashMap<String, String>>,
    prefix: Option<String>,
    suffix: Option<String>,
) -> Result<CodegenUri> {
    let param_map = params.unwrap_or_default();
    let mut resolved_path = route_path;
    let mut query_params: Vec<(String, String)> = Vec::new();

    let (path_part, query_part) = match resolved_path.split_once('?') {
        Some((p, q)) => (p.to_string(), Some(q.to_string())),
        None => (resolved_path.clone(), None),
    };

    resolved_path = path_part;

    for (key, val) in &param_map {
        let pattern1 = format!("<{key}>");
        let pattern2 = format!(":{key}");
        let pattern3 = format!("<{key}..>");
        let encoded_val = url_encode(val);

        if resolved_path.contains(&pattern1) {
            resolved_path = resolved_path.replace(&pattern1, &encoded_val);
        } else if resolved_path.contains(&pattern3) {
            resolved_path = resolved_path.replace(&pattern3, &encoded_val);
        } else if resolved_path.contains(&pattern2) {
            resolved_path = resolved_path.replace(&pattern2, &encoded_val);
        } else {
            query_params.push((key.clone(), encoded_val));
        }
    }

    let mut final_query = String::new();
    if let Some(qp) = query_part {
        final_query.push_str(&qp);
    }
    for (k, v) in query_params {
        if !final_query.is_empty() {
            final_query.push('&');
        }
        final_query.push_str(&format!("{k}={v}"));
    }

    let mut full_uri = String::new();
    if let Some(p) = prefix {
        full_uri.push_str(p.trim_end_matches('/'));
    }
    full_uri.push_str(&resolved_path);
    if !final_query.is_empty() {
        full_uri.push('?');
        full_uri.push_str(&final_query);
    }
    if let Some(s) = suffix {
        full_uri.push_str(&s);
    }

    let result = CodegenUri {
        uri: full_uri,
        path: resolved_path,
        query: if final_query.is_empty() {
            None
        } else {
            Some(final_query)
        },
        params: param_map,
    };

    debug_log(&format!("Generated URI: {}", result.uri));
    Ok(result)
}

/// Generate TypeScript type declaration definitions for routes.
#[napi]
pub fn generate_ts_declarations(routes: Vec<CodegenRoute>) -> String {
    let mut out = String::new();
    out.push_str("// Auto-generated by Rocket NAPI Codegen\n\n");
    out.push_str("export interface RocketRoutes {\n");

    for r in &routes {
        out.push_str(&format!("  \"{} {}\": {{\n", r.method, r.path));
        out.push_str("    params: {\n");
        for p in &r.params {
            out.push_str(&format!("      {}: {};\n", p.name, p.ts_type));
        }
        out.push_str("    };\n");
        out.push_str("  };\n");
    }

    out.push_str("}\n");

    debug_log("Generated TypeScript declarations");
    out
}

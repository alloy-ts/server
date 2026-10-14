//! NAPI bindings for Rocket route URI construction, route pattern parsing,
//! and TypeScript route type generation.

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
            if let Some(name) = inner.strip_suffix("..") {
                path_params.push(RouteParam {
                    name: name.to_string(),
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
    }
}

#[napi]
pub fn uri(pattern: String, params: Option<HashMap<String, serde_json::Value>>) -> String {
    let params = match params {
        Some(p) => p,
        None => return pattern,
    };

    let mut result = pattern;
    for (key, val) in params {
        let val_str = match val {
            serde_json::Value::String(s) => s,
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Null => String::new(),
            other => serde_json::to_string(&other).unwrap_or_default(),
        };
        let encoded = urlencoding_encode(&val_str);

        result = result
            .replace(&format!(":{key}"), &encoded)
            .replace(&format!("<{key}..>"), &encoded)
            .replace(&format!("<{key}>"), &encoded);
    }
    result
}

#[napi]
pub fn build_route_uri(
    pattern: String,
    params: Option<HashMap<String, serde_json::Value>>,
) -> String {
    let params = match params {
        Some(p) => p,
        None => return pattern,
    };

    let (path_part, query_part) = match pattern.split_once('?') {
        Some((p, q)) => (p, Some(q)),
        None => (pattern.as_str(), None),
    };

    let path = uri(path_part.to_string(), Some(params.clone()));

    let mut result_path = path;

    if let Some(q) = query_part {
        let mut query_pairs = Vec::new();
        for seg in q.split('&').filter(|s| !s.is_empty()) {
            let mut param_name = seg;
            if let Some(name) = seg.strip_prefix(':') {
                param_name = name;
            } else if seg.starts_with('<') && seg.ends_with('>') {
                let inner = &seg[1..seg.len() - 1];
                param_name = inner.strip_suffix("..").unwrap_or(inner);
            } else if let Some((key, _)) = seg.split_once('=') {
                param_name = key;
            }

            if let Some(val) = params.get(param_name) {
                let val_str = match val {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Null => String::new(),
                    other => serde_json::to_string(&other).unwrap_or_default(),
                };
                query_pairs.push(format!(
                    "{}={}",
                    urlencoding_encode(param_name),
                    urlencoding_encode(&val_str)
                ));
            }
        }
        if !query_pairs.is_empty() {
            result_path.push('?');
            result_path.push_str(&query_pairs.join("&"));
        }
    }

    result_path
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
pub fn generate_ts_route_types(patterns: Vec<String>) -> String {
    let mut code = String::from("// Generated Rocket Route Types\n\n");
    code.push_str("export interface ApiRoutes {\n");

    for pattern in patterns {
        let route = parse_route_pattern(pattern);
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

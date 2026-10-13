//! # NAPI bindings for Rocket Code Generation Utilities.
//!
//! Provides route parsing, syntax translation (Rocket -> Alloy/Bun),
//! client SDK generation, route handler boilerplate generation,
//! and TypeScript interface generation.

use napi_derive::napi;

/// A dynamic parameter extracted from a Rocket route pattern.
#[napi(object)]
#[derive(Clone, Debug, PartialEq)]
pub struct RouteParam {
    pub name: String,
    pub param_type: String,
    pub is_path: bool,
    pub is_query: bool,
    pub is_optional: bool,
}

/// Specification of a Rocket route used for client and handler generation.
#[napi(object)]
#[derive(Clone, Debug)]
pub struct RouteSpec {
    pub method: String,
    pub path: String,
    pub name: Option<String>,
    pub params: Option<Vec<RouteParam>>,
    pub return_type: Option<String>,
}

/// Options for client SDK code generation.
#[napi(object)]
#[derive(Clone, Debug)]
pub struct ClientOptions {
    pub client_name: Option<String>,
    pub base_url: Option<String>,
}

/// Specification of a field for TypeScript interface generation.
#[napi(object)]
#[derive(Clone, Debug)]
pub struct FieldSpec {
    pub name: String,
    pub r#type: String,
    pub optional: Option<bool>,
}

/// The generated code output.
#[napi(object)]
#[derive(Clone, Debug)]
pub struct CodegenResult {
    pub code: String,
}

/// Parse dynamic path and query parameters from a Rocket route pattern.
///
/// Example pattern: `/users/<id>?<age>&<name..>`
#[napi]
pub fn parse_rocket_route_pattern(pattern: String) -> Vec<RouteParam> {
    let mut params = Vec::new();
    let parts: Vec<&str> = pattern.splitn(2, '?').collect();
    let path_part = parts[0];
    let query_part = parts.get(1).copied();

    // Parse path parameters
    for seg in path_part.split('/') {
        if seg.is_empty() {
            continue;
        }
        if let Some(start) = seg.find('<') {
            if let Some(end) = seg.find('>') {
                if end > start {
                    let inner = &seg[start + 1..end];
                    if inner == "_" || inner == "_.." {
                        continue;
                    }
                    let is_optional = inner.ends_with("..");
                    let name = if is_optional {
                        &inner[..inner.len() - 2]
                    } else {
                        inner
                    };
                    params.push(RouteParam {
                        name: name.to_string(),
                        param_type: "string".to_string(),
                        is_path: true,
                        is_query: false,
                        is_optional,
                    });
                }
            }
        }
    }

    // Parse query parameters
    if let Some(query_str) = query_part {
        for item in query_str.split('&') {
            if item.is_empty() {
                continue;
            }
            if let Some(start) = item.find('<') {
                if let Some(end) = item.find('>') {
                    if end > start {
                        let inner = &item[start + 1..end];
                        if inner == "_" || inner == "_.." {
                            continue;
                        }
                        let is_optional = inner.ends_with("..");
                        let name = if is_optional {
                            &inner[..inner.len() - 2]
                        } else {
                            inner
                        };
                        params.push(RouteParam {
                            name: name.to_string(),
                            param_type: "string".to_string(),
                            is_path: false,
                            is_query: true,
                            is_optional,
                        });
                    }
                }
            }
        }
    }

    params
}

/// Translate Rocket route syntax (`/users/<id>/<rest..>`) into Alloy/Bun syntax (`/users/:id/*`).
#[napi]
pub fn translate_pattern_to_alloy(pattern: String) -> String {
    let mut out = String::with_capacity(pattern.len());
    let mut chars = pattern.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '<' {
            let mut inner = String::new();
            while let Some(&nc) = chars.peek() {
                if nc == '>' {
                    chars.next();
                    break;
                }
                inner.push(nc);
                chars.next();
            }
            if inner.ends_with("..") {
                out.push('*');
            } else if inner != "_" {
                out.push(':');
                out.push_str(&inner);
            }
        } else {
            out.push(c);
        }
    }

    out
}

/// Generate a strongly-typed TypeScript fetch client SDK for Rocket route specifications.
#[napi]
pub fn generate_typescript_client(
    routes: Vec<RouteSpec>,
    options: Option<ClientOptions>,
) -> CodegenResult {
    let client_name = options
        .as_ref()
        .and_then(|o| o.client_name.clone())
        .unwrap_or_else(|| "ApiClient".to_string());
    let base_url = options
        .as_ref()
        .and_then(|o| o.base_url.clone())
        .unwrap_or_default();

    let mut code = String::new();
    code.push_str(&format!("export class {} {{\n", client_name));
    code.push_str("  private baseUrl: string;\n\n");
    code.push_str(&format!(
        "  constructor(baseUrl: string = \"{}\") {{\n",
        base_url
    ));
    code.push_str("    this.baseUrl = baseUrl;\n");
    code.push_str("  }\n\n");

    for route in routes {
        let method_lower = route.method.to_lowercase();
        let method_upper = route.method.to_uppercase();
        let name = route.name.clone().unwrap_or_else(|| {
            let clean_path = route
                .path
                .replace(['/', '<', '>', '.', '?', '&', '=', ':'], "_");
            format!("{}{}", method_lower, clean_path)
        });

        let params = route
            .params
            .clone()
            .unwrap_or_else(|| parse_rocket_route_pattern(route.path.clone()));

        let return_type = route
            .return_type
            .clone()
            .unwrap_or_else(|| "Promise<unknown>".to_string());

        // Build function parameters signature
        let mut fn_args = Vec::new();
        for p in &params {
            let opt_flag = if p.is_optional { "?" } else { "" };
            fn_args.push(format!("{}{}: {}", p.name, opt_flag, p.param_type));
        }

        code.push_str(&format!(
            "  async {}({}): {} {{\n",
            name,
            fn_args.join(", "),
            return_type
        ));

        // Build path construction
        let mut path_code = route.path.clone();
        for p in &params {
            if p.is_path {
                let target_rocket = format!("<{}>", p.name);
                let target_rocket_tail = format!("<{}..>", p.name);
                path_code = path_code.replace(&target_rocket, &format!("${{{}}}", p.name));
                path_code = path_code.replace(&target_rocket_tail, &format!("${{{}}}", p.name));
            }
        }

        // Handle query parameters
        let query_params: Vec<&RouteParam> = params.iter().filter(|p| p.is_query).collect();

        if query_params.is_empty() {
            code.push_str(&format!(
                "    const url = `${{this.baseUrl}}{}`;\n",
                path_code
            ));
        } else {
            let base_path_only = path_code.split('?').next().unwrap_or(&path_code);
            code.push_str(&format!(
                "    const url = new URL(`${{this.baseUrl}}{}`);\n",
                base_path_only
            ));
            for qp in &query_params {
                if qp.is_optional {
                    code.push_str(&format!(
                        "    if ({} !== undefined) url.searchParams.append(\"{}\", String({}));\n",
                        qp.name, qp.name, qp.name
                    ));
                } else {
                    code.push_str(&format!(
                        "    url.searchParams.append(\"{}\", String({}));\n",
                        qp.name, qp.name
                    ));
                }
            }
        }

        let url_expr = if query_params.is_empty() {
            "url"
        } else {
            "url.toString()"
        };

        code.push_str(&format!(
            "    const response = await fetch({}, {{\n",
            url_expr
        ));
        code.push_str(&format!("      method: \"{}\",\n", method_upper));
        code.push_str("    });\n");
        code.push_str("    if (!response.ok) {\n");
        code.push_str(
            "      throw new Error(`HTTP error ${response.status}: ${response.statusText}`);\n",
        );
        code.push_str("    }\n");
        code.push_str("    return response.json();\n");
        code.push_str("  }\n\n");
    }

    code.push_str("}\n");

    CodegenResult { code }
}

/// Generate TypeScript route handler boilerplate for a given Rocket route.
#[napi]
pub fn generate_route_handler(route: RouteSpec) -> CodegenResult {
    let name = route.name.unwrap_or_else(|| {
        format!(
            "{}Handler",
            route.method.to_lowercase()
        )
    });
    let params = route
        .params
        .unwrap_or_else(|| parse_rocket_route_pattern(route.path.clone()));

    let mut code = String::new();
    code.push_str(&format!(
        "export async function {}(req: any): Promise<Response> {{\n",
        name
    ));

    if !params.is_empty() {
        let param_names: Vec<String> = params.into_iter().map(|p| p.name).collect();
        code.push_str(&format!(
            "  const {{ {} }} = req.params || {{}};\n",
            param_names.join(", ")
        ));
    }

    code.push_str(&format!(
        "  return new Response(`Handled {} {}`);\n",
        route.method.to_uppercase(),
        route.path
    ));
    code.push_str("}\n");

    CodegenResult { code }
}

/// Convert field definitions into a TypeScript interface definition.
#[napi]
pub fn generate_typescript_interface(name: String, fields: Vec<FieldSpec>) -> CodegenResult {
    let mut code = String::new();
    code.push_str(&format!("export interface {} {{\n", name));
    for field in fields {
        let opt_flag = if field.optional.unwrap_or(false) {
            "?"
        } else {
            ""
        };
        code.push_str(&format!(
            "  {}{}: {};\n",
            field.name, opt_flag, field.r#type
        ));
    }
    code.push_str("}\n");

    CodegenResult { code }
}

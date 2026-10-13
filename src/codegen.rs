//! NAPI bindings for Rocket TypeScript code generation utilities.

use napi_derive::napi;

#[napi(object)]
#[derive(Debug, Clone)]
pub struct RouteParam {
    pub name: String,
    pub param_type: String,
    pub is_path: bool,
    pub is_query: bool,
    pub is_optional: bool,
}

#[napi(object)]
#[derive(Debug, Clone)]
pub struct RouteSpec {
    pub method: String,
    pub path: String,
    pub name: String,
    pub params: Vec<RouteParam>,
    pub return_type: Option<String>,
}

#[napi(object)]
#[derive(Debug, Clone)]
pub struct ClientOptions {
    pub client_name: Option<String>,
    pub base_url: Option<String>,
}

#[napi(object)]
#[derive(Debug, Clone)]
pub struct CodegenOutput {
    pub code: String,
}

/// Parse dynamic path and query parameters from a Rocket route pattern string.
///
/// Example: `"/users/<id>?<age>&<name..>"`
#[napi]
pub fn parse_rocket_route_pattern(pattern: String) -> Vec<RouteParam> {
    let mut params = Vec::new();
    let parts: Vec<&str> = pattern.splitn(2, '?').collect();
    let path_part = parts[0];
    let query_part = parts.get(1).copied();

    // Parse path parameters
    for seg in path_part.split('/') {
        if seg.starts_with('<') && seg.ends_with('>') {
            let inner = &seg[1..seg.len() - 1];
            if inner == "_" || inner == "_.." {
                continue;
            }
            if let Some(name) = inner.strip_suffix("..") {
                if name != "_" {
                    params.push(RouteParam {
                        name: name.to_string(),
                        param_type: "string".to_string(),
                        is_path: true,
                        is_query: false,
                        is_optional: true,
                    });
                }
            } else {
                let (name, ptype) = parse_typed_param(inner);
                if name != "_" {
                    params.push(RouteParam {
                        name,
                        param_type: ptype,
                        is_path: true,
                        is_query: false,
                        is_optional: false,
                    });
                }
            }
        }
    }

    // Parse query parameters
    if let Some(qp) = query_part {
        for item in qp.split('&') {
            if item.is_empty() {
                continue;
            }
            if item.starts_with('<') && item.ends_with('>') {
                let inner = &item[1..item.len() - 1];
                if inner == "_" || inner == "_.." {
                    continue;
                }
                if let Some(name) = inner.strip_suffix("..") {
                    if name != "_" {
                        params.push(RouteParam {
                            name: name.to_string(),
                            param_type: "string".to_string(),
                            is_path: false,
                            is_query: true,
                            is_optional: true,
                        });
                    }
                } else {
                    let (name, ptype) = parse_typed_param(inner);
                    if name != "_" {
                        params.push(RouteParam {
                            name,
                            param_type: ptype,
                            is_path: false,
                            is_query: true,
                            is_optional: false,
                        });
                    }
                }
            } else if item.contains('=') {
                let kv: Vec<&str> = item.splitn(2, '=').collect();
                let key = kv[0];
                let val = kv[1];
                if val.starts_with('<') && val.ends_with('>') {
                    let inner = &val[1..val.len() - 1];
                    let (_, ptype) = parse_typed_param(inner);
                    params.push(RouteParam {
                        name: key.to_string(),
                        param_type: ptype,
                        is_path: false,
                        is_query: true,
                        is_optional: false,
                    });
                }
            }
        }
    }

    params
}

fn parse_typed_param(inner: &str) -> (String, String) {
    if let Some((name, ty)) = inner.split_once(':') {
        let name = name.trim().to_string();
        let ty = rust_type_to_ts(ty.trim());
        (name, ty)
    } else {
        (inner.trim().to_string(), "string".to_string())
    }
}

fn rust_type_to_ts(rust_ty: &str) -> String {
    match rust_ty {
        "u8" | "u16" | "u32" | "u64" | "usize" | "i8" | "i16" | "i32" | "i64" | "isize"
        | "f32" | "f64" => "number".to_string(),
        "bool" => "boolean".to_string(),
        "String" | "&str" | "str" | "PathBuf" => "string".to_string(),
        other => {
            if let Some(inner) = other.strip_prefix("Option<").and_then(|s| s.strip_suffix('>')) {
                format!("{} | null", rust_type_to_ts(inner.trim()))
            } else if let Some(inner) = other.strip_prefix("Vec<").and_then(|s| s.strip_suffix('>')) {
                format!("{}[]", rust_type_to_ts(inner.trim()))
            } else {
                other.to_string()
            }
        }
    }
}

/// Translate Rocket route pattern syntax (`/users/<id>/<rest..>`) into Alloy/Bun syntax (`/users/:id/*`).
#[napi]
pub fn translate_pattern_to_alloy(pattern: String) -> String {
    let path_part = pattern.split('?').next().unwrap_or(&pattern);
    let mut segments: Vec<String> = Vec::new();

    for seg in path_part.split('/') {
        if seg.is_empty() {
            if segments.is_empty() {
                segments.push("".to_string());
            }
            continue;
        }
        if seg.starts_with('<') && seg.ends_with('>') {
            let inner = &seg[1..seg.len() - 1];
            if inner.ends_with("..") || inner == "_.." {
                segments.push("*".to_string());
            } else {
                let name = inner.split(':').next().unwrap_or(inner);
                segments.push(format!(":{name}"));
            }
        } else {
            segments.push(seg.to_string());
        }
    }

    let mut res = segments.join("/");
    if res.is_empty() {
        res = "/".to_string();
    }
    if !res.starts_with('/') {
        res = format!("/{res}");
    }
    res
}

/// Generate a strongly-typed TypeScript client SDK for Rocket route specifications.
#[napi]
pub fn generate_typescript_client(routes: Vec<RouteSpec>, options: Option<ClientOptions>) -> CodegenOutput {
    let client_name = options
        .as_ref()
        .and_then(|o| o.client_name.clone())
        .unwrap_or_else(|| "ApiClient".to_string());
    let base_url = options
        .as_ref()
        .and_then(|o| o.base_url.clone())
        .unwrap_or_else(|| "".to_string());

    let mut code = String::new();
    code.push_str(&format!("export class {client_name} {{\n"));
    code.push_str("  private baseUrl: string;\n\n");
    code.push_str("  constructor(baseUrl: string = ");
    code.push_str(&format!("{base_url:?}) {{\n"));
    code.push_str("    this.baseUrl = baseUrl;\n");
    code.push_str("  }\n\n");

    for route in routes {
        let return_type = route.return_type.as_deref().unwrap_or("Promise<unknown>");
        let mut fn_args = Vec::new();

        let path_params: Vec<&RouteParam> = route.params.iter().filter(|p| p.is_path).collect();
        let query_params: Vec<&RouteParam> = route.params.iter().filter(|p| p.is_query).collect();

        for p in &path_params {
            fn_args.push(format!("{}: {}", p.name, p.param_type));
        }
        for p in &query_params {
            let opt = if p.is_optional { "?" } else { "" };
            fn_args.push(format!("{}{}: {}", p.name, opt, p.param_type));
        }

        let method_name = if route.name.is_empty() {
            format!("{}_{}", route.method.to_lowercase(), route.path.replace('/', "_"))
        } else {
            route.name.clone()
        };

        code.push_str(&format!("  async {method_name}({}): {return_type} {{\n", fn_args.join(", ")));

        // Build path URL template
        let alloy_path = translate_pattern_to_alloy(route.path.clone());
        let mut url_expr = alloy_path;
        for p in &path_params {
            url_expr = url_expr.replace(&format!(":{}", p.name), &format!("${{{}}}", p.name));
            url_expr = url_expr.replace("*", &format!("${{{}}}", p.name));
        }

        code.push_str(&format!("    let url = `${{this.baseUrl}}{url_expr}`;\n"));

        if !query_params.is_empty() {
            code.push_str("    const query = new URLSearchParams();\n");
            for p in &query_params {
                code.push_str(&format!("    if ({} !== undefined && {} !== null) {{\n", p.name, p.name));
                code.push_str(&format!("      query.append({:?}, String({}));\n", p.name, p.name));
                code.push_str("    }\n");
            }
            code.push_str("    if (query.toString()) {\n");
            code.push_str("      url += `?${query.toString()}`;\n");
            code.push_str("    }\n");
        }

        code.push_str(&format!("    const res = await fetch(url, {{ method: {:?} }});\n", route.method.to_uppercase()));
        code.push_str("    return await res.json();\n");
        code.push_str("  }\n\n");
    }

    code.push_str("}\n");
    CodegenOutput { code }
}

/// Generate TypeScript route handler boilerplate for a given Rocket route specification.
#[napi]
pub fn generate_route_handler(route: RouteSpec) -> CodegenOutput {
    let handler_name = if route.name.is_empty() {
        "handleRequest".to_string()
    } else {
        format!("handle{}", uppercase_first(&route.name))
    };

    let mut code = String::new();
    code.push_str("import { RequestInput, Response, responseJson } from \"@alloy-ts/server\";\n\n");
    code.push_str(&format!("export async function {handler_name}(req: RequestInput): Promise<Response> {{\n"));

    let path_params: Vec<&RouteParam> = route.params.iter().filter(|p| p.is_path).collect();
    if !path_params.is_empty() {
        let names: Vec<String> = path_params.iter().map(|p| p.name.clone()).collect();
        code.push_str(&format!("  const {{ {} }} = req.params;\n", names.join(", ")));
    }

    code.push_str("  // TODO: Implement handler logic\n");
    code.push_str("  return responseJson({ status: \"ok\" });\n");
    code.push_str("}\n");

    CodegenOutput { code }
}

fn uppercase_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Convert a Rust struct or enum definition string into a TypeScript interface or type definition.
#[napi]
pub fn generate_typescript_interface(rust_def: String) -> CodegenOutput {
    let mut code = String::new();
    let lines: Vec<&str> = rust_def.lines().map(|l| l.trim()).collect();

    let mut in_struct = false;
    let mut in_enum = false;
    let mut name = String::new();
    let mut fields = Vec::new();
    let mut enum_variants = Vec::new();

    for line in lines {
        if line.starts_with("pub struct ") || line.starts_with("struct ") {
            in_struct = true;
            let parts: Vec<&str> = line.split_whitespace().collect();
            let idx = if line.starts_with("pub ") { 2 } else { 1 };
            if idx < parts.len() {
                name = parts[idx].trim_matches('{').to_string();
            }
        } else if line.starts_with("pub enum ") || line.starts_with("enum ") {
            in_enum = true;
            let parts: Vec<&str> = line.split_whitespace().collect();
            let idx = if line.starts_with("pub ") { 2 } else { 1 };
            if idx < parts.len() {
                name = parts[idx].trim_matches('{').to_string();
            }
        } else if line == "}" {
            break;
        } else if in_struct {
            if line.contains(':') {
                let kv: Vec<&str> = line.splitn(2, ':').collect();
                let field_name = kv[0].trim().trim_start_matches("pub ");
                let field_type = kv[1].trim().trim_matches(',').trim();
                fields.push((field_name.to_string(), field_type.to_string()));
            }
        } else if in_enum {
            let variant = line.trim_matches(',').trim();
            if !variant.is_empty() && !variant.starts_with("//") {
                enum_variants.push(variant.to_string());
            }
        }
    }

    if in_struct {
        code.push_str(&format!("export interface {name} {{\n"));
        for (f_name, f_type) in fields {
            let ts_name = snake_to_camel(&f_name);
            if let Some(inner) = f_type.strip_prefix("Option<").and_then(|s| s.strip_suffix('>')) {
                let ts_type = rust_type_to_ts(inner.trim());
                code.push_str(&format!("  {ts_name}?: {ts_type};\n"));
            } else {
                let ts_type = rust_type_to_ts(&f_type);
                code.push_str(&format!("  {ts_name}: {ts_type};\n"));
            }
        }
        code.push_str("}\n");
    } else if in_enum {
        code.push_str(&format!("export type {name} =\n"));
        for (i, v) in enum_variants.iter().enumerate() {
            let sep = if i == enum_variants.len() - 1 { ";" } else { "" };
            code.push_str(&format!("  | {:?}{}\n", v, sep));
        }
    } else {
        code.push_str("// Unable to parse Rust definition\n");
    }

    CodegenOutput { code }
}

fn snake_to_camel(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize = false;
    for c in s.chars() {
        if c == '_' {
            capitalize = true;
        } else if capitalize {
            result.push(c.to_ascii_uppercase());
            capitalize = false;
        } else {
            result.push(c);
        }
    }
    result
}

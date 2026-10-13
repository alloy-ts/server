//! # Codegen — Rocket's code generation, re-expressed for TypeScript
//!
//! Rocket's own code generation lives in the `rocket_codegen` crate and is
//! implemented entirely with **procedural macros** (`#[get]`, `routes!`,
//! `uri!`, `derive(FromForm)`, …). Those macros run inside `rustc` and emit
//! Rust items — there is no runtime API to call and nothing that can cross the
//! N-API boundary.
//!
//! This module is the N-API-facing counterpart. It performs, at *runtime*, the
//! job the macros perform at compile time, with JavaScript on the other side:
//!
//! | `rocket_codegen` (compile time)       | this module (runtime)                        |
//! |---------------------------------------|---------------------------------------------|
//! | `#[get("/<id>")]` parses a route URI  | [`parse_route`] parses a route pattern      |
//! | `routes!` collects route metadata     | [`parse_routes`] builds `Vec<`[`RouteDef]`>`|
//! | type-checked `uri!(route(a, b))` URIs | the generated `uri()` builder                |
//! | `FromForm` / `Responder` derives      | generated `ParamsOf` / `QueryOf` / … aliases|
//!
//! [`generate_types`], [`generate_client`], and [`generate_module`] return
//! **TypeScript source code** as a string, so a Node.js caller can write a
//! fully typed API surface straight from the route table it already passes to
//! [`crate::server::serve`]:
//!
//! ```ts
//! import { parseRoutes, generateModule } from "@alloy-ts/server";
//! import { writeFileSync } from "node:fs";
//!
//! const routes = parseRoutes([
//!   { method: "GET",  path: "/users/:id", response: "User" },
//!   { method: "POST", path: "/users",    body: "CreateUser", response: "User" },
//! ]);
//!
//! writeFileSync("api.ts", generateModule(routes, { base_url: "http://localhost:3000" }));
//! ```
//!
//! # JavaScript field names
//!
//! napi-rs maps the Rust `snake_case` field names declared here to `camelCase`
//! in JavaScript: `param_types` is `paramTypes`, `query_types` is
//! `queryTypes`, `ts_name` is `tsName`, `ts_type` is `tsType`, `base_url` is
//! `baseUrl`, `client_name` is `clientName`, `types_name` is `typesName`,
//! `server_import` is `serverImport`, and `types_import` is `typesImport`.
//! Unknown keys are *ignored* rather than rejected, so a mistyped option
//! silently falls back to its default.
//!
//! # Pattern syntax
//!
//! Both the Alloy/Bun syntax used by [`crate::server::serve`] and Rocket's own
//! route syntax are accepted, so the same string can be handed to `serve()` and
//! to this module:
//!
//! | pattern          | meaning                             |
//! |------------------|-------------------------------------|
//! | `:name` / `<name>`   | a single dynamic path segment    |
//! | `*` / `<name..>` / `<_..>` | trailing, multi-segment match |
//! | `?a&b` / `?<a>&<b>`  | query parameters                 |
//!
//! Unknown method names are rejected up front (mirroring Rocket's route
//! attribute, which only accepts the HTTP methods Rocket knows).

use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;

use napi::bindgen_prelude::Error as NapiError;
use napi::Status;
use napi_derive::napi;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// The HTTP methods Rocket accepts in a route attribute, in declaration order.
pub const ALL_METHODS: &[&str] = &[
    "GET", "PUT", "POST", "DELETE", "OPTIONS", "HEAD", "PATCH", "TRACE", "CONNECT",
];

/// Where a dynamic route parameter comes from.
#[napi(string_enum)]
#[derive(Clone, Copy, Debug)]
pub enum ParamKind {
    /// A dynamic path segment (`:id`, `<id>`).
    Path,
    /// A query parameter (`?id`, `?<id>`).
    Query,
    /// The request body.
    Body,
}

/// One dynamic parameter of a [`RouteDef`].
#[napi(object)]
#[derive(Clone, Debug)]
pub struct RouteParam {
    /// The parameter name as written in the pattern (`id`).
    pub name: String,
    /// A TypeScript-safe version of `name` (`type` becomes `type_`).
    pub ts_name: String,
    pub kind: ParamKind,
    /// The TypeScript type emitted for this parameter.
    pub ts_type: String,
    /// `true` for multi-segment params (`<name..>`, `*`).
    pub trailing: bool,
    /// `true` when the parameter may be omitted.
    pub optional: bool,
}

/// Overrides the TypeScript type of one path or query parameter.
#[napi(object)]
#[derive(Clone, Debug)]
pub struct TypeOverride {
    /// Parameter name as it appears in the pattern.
    pub name: String,
    /// TypeScript type expression, e.g. `"number"` or `"\"a\" | \"b\""`.
    pub ts_type: String,
}

/// A route as written by the caller — the runtime analogue of a `#[get(..)]`
/// decorated function plus its `routes!` entry.
#[napi(object)]
#[derive(Clone, Debug)]
pub struct RouteInput {
    /// HTTP method (`"GET"`, `"post"`, …); case-insensitive.
    pub method: String,
    /// Route pattern (`"/users/:id"`, `"/users/<id..>?<q>"`, …).
    pub path: String,
    /// Optional explicit handler name; derived from method + path when absent.
    pub name: Option<String>,
    /// TypeScript type of the request body, e.g. `"CreateUser"`.
    pub body: Option<String>,
    /// TypeScript type of the response body, e.g. `"User"`.
    pub response: Option<String>,
    /// Per-parameter type overrides for path params.
    pub param_types: Option<Vec<TypeOverride>>,
    /// Per-parameter type overrides for query params.
    pub query_types: Option<Vec<TypeOverride>>,
}

/// A fully resolved route definition — what `routes!` would have baked into a
/// static `Route`, kept in a shape the codegen layer can emit TypeScript from.
#[napi(object)]
#[derive(Clone, Debug)]
pub struct RouteDef {
    /// `"GET /users/:id"` — method plus the normalized path.
    pub key: String,
    /// Upper-cased HTTP method.
    pub method: String,
    /// Normalized path in `:param` form (`"/users/:id"`).
    pub path: String,
    /// The same pattern in Rocket's `<param>` form (`"/users/<id>"`).
    pub rocket_path: String,
    /// Generated handler name (`"getUserById"`).
    pub name: String,
    /// Path parameters, in declaration order.
    pub params: Vec<RouteParam>,
    /// Query parameters, in declaration order.
    pub query: Vec<RouteParam>,
    /// Request body type, if declared.
    pub body: Option<String>,
    /// Response body type, if declared.
    pub response: Option<String>,
}

/// Options controlling the emitted TypeScript.
#[napi(object)]
#[derive(Clone, Debug, Default)]
pub struct CodegenOptions {
    /// Name of the generated client class (default `"RocketClient"`).
    pub client_name: Option<String>,
    /// Name of the generated route map interface (default `"RouteTypes"`).
    pub types_name: Option<String>,
    /// Default `baseUrl` baked into the generated client (default `""`).
    pub base_url: Option<String>,
    /// Module to import `RequestInput` from; a local mirror is emitted if unset.
    pub server_import: Option<String>,
    /// Module to import the generated types from (client output only).
    pub types_import: Option<String>,
    /// Emit the "do not edit" banner (default `true`).
    pub banner: Option<bool>,
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

/// Rust/TypeScript reserved words that cannot be used as a parameter name.
const RESERVED: &[&str] = &[
    "as", "async", "await", "break", "case", "catch", "class", "const", "continue", "debugger",
    "default", "delete", "do", "else", "enum", "export", "extends", "false", "finally", "for",
    "from", "function", "get", "if", "implements", "import", "in", "instanceof", "interface", "let",
    "module", "new", "null", "of", "package", "private", "protected", "public", "require", "return",
    "set", "static", "super", "switch", "this", "throw", "true", "try", "type", "typeof", "var",
    "void", "while", "with", "yield",
];

struct Pattern {
    ts_path: String,
    rocket_path: String,
    path_params: Vec<(String, bool)>,
    query: Vec<String>,
}

fn is_ident(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn sanitize_ident(name: &str, index: usize) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' })
        .collect();
    let cleaned = if cleaned.is_empty() || cleaned.starts_with(|c: char| c.is_ascii_digit()) {
        format!("param{index}_{cleaned}")
    } else {
        cleaned
    };
    if RESERVED.contains(&cleaned.as_str()) {
        format!("{cleaned}_")
    } else {
        cleaned
    }
}

/// `<name>` / `<name..>` -> `Some("name")` / `Some("name..")`.
fn rocket_segment(segment: &str) -> Option<&str> {
    segment.strip_prefix('<').and_then(|s| s.strip_suffix('>'))
}

fn parse_pattern(pattern: &str) -> Pattern {
    let (path_part, query_part) = match pattern.split_once('?') {
        Some((path, query)) => (path, Some(query)),
        None => (pattern, None),
    };

    let mut ts_path = String::new();
    let mut rocket_path = String::new();
    let mut path_params: Vec<(String, bool)> = Vec::new();

    for segment in path_part.split('/') {
        if segment.is_empty() {
            continue;
        }
        ts_path.push('/');
        rocket_path.push('/');

        if segment == "*" {
            // Bun/Alloy wildcard: any number of trailing segments.
            ts_path.push('*');
            rocket_path.push_str("<_..>");
            path_params.push(("_".to_string(), true));
        } else if let Some(name) = segment.strip_prefix(':') {
            if is_ident(name) {
                ts_path.push(':');
                ts_path.push_str(name);
                rocket_path.push('<');
                rocket_path.push_str(name);
                rocket_path.push('>');
                path_params.push((name.to_string(), false));
            } else {
                ts_path.push_str(segment);
                rocket_path.push_str(segment);
            }
        } else if let Some(inner) = rocket_segment(segment) {
            let (name, trailing) = match inner.strip_suffix("..") {
                Some(name) => (name, true),
                None => (inner, false),
            };
            if name == "_" {
                ts_path.push('*');
                rocket_path.push_str("<_..>");
                path_params.push(("_".to_string(), true));
            } else if is_ident(name) {
                rocket_path.push('<');
                rocket_path.push_str(name);
                rocket_path.push_str(if trailing { "..>" } else { ">" });
                ts_path.push(':');
                ts_path.push_str(name);
                path_params.push((name.to_string(), trailing));
            } else {
                ts_path.push_str(segment);
                rocket_path.push_str(segment);
            }
        } else {
            ts_path.push_str(segment);
            rocket_path.push_str(segment);
        }
    }

    if ts_path.is_empty() {
        ts_path.push('/');
    }
    if rocket_path.is_empty() {
        rocket_path.push('/');
    }

    let mut query = Vec::new();
    if let Some(query_part) = query_part {
        for raw in query_part.split('&') {
            if raw.is_empty() {
                continue;
            }
            let name = rocket_segment(raw).unwrap_or(raw);
            let name = name.strip_suffix("..").unwrap_or(name);
            if name == "_" {
                continue;
            }
            query.push(sanitize_ident(name, query.len()));
        }
    }

    Pattern {
        ts_path,
        rocket_path,
        path_params,
        query,
    }
}

fn override_map(overrides: &Option<Vec<TypeOverride>>) -> HashMap<String, String> {
    overrides
        .as_ref()
        .map(|list| {
            list.iter()
                .map(|o| (o.name.clone(), o.ts_type.clone()))
                .collect()
        })
        .unwrap_or_default()
}

/// Derive a handler name from a method and path: `GET /users/:id` ->
/// `getUserById`.
fn derive_method_name(method: &str, ts_path: &str) -> String {
    let mut out = method.to_ascii_lowercase();
    let mut capitalize = true;
    for c in ts_path.chars() {
        match c {
            '/' | '-' | '_' | '.' => capitalize = true,
            ':' => {
                out.push_str("By");
                capitalize = true;
            }
            '*' => {}
            c if c.is_ascii_alphanumeric() => {
                if capitalize {
                    out.push(c.to_ascii_uppercase());
                    capitalize = false;
                } else {
                    out.push(c);
                }
            }
            _ => {}
        }
    }
    if out.starts_with(|c: char| c.is_ascii_digit()) {
        out.insert(0, '_');
    }
    out
}

/// Make every `name` in `defs` unique, suffixing collisions with `_2`, `_3`, …
fn dedupe_names(defs: &mut [RouteDef]) {
    let mut seen: HashSet<String> = HashSet::new();
    for def in defs.iter_mut() {
        if seen.contains(&def.name) {
            let base = def.name.clone();
            let mut suffix = 2;
            while seen.contains(&format!("{base}_{suffix}")) {
                suffix += 1;
            }
            def.name = format!("{base}_{suffix}");
        }
        seen.insert(def.name.clone());
    }
}

// ---------------------------------------------------------------------------
// NAPI entry points
// ---------------------------------------------------------------------------

/// The HTTP methods Rocket route attributes accept.
#[napi]
pub fn http_methods() -> Vec<String> {
    ALL_METHODS.iter().map(|m| m.to_string()).collect()
}

/// Resolve one route pattern into a [`RouteDef`].
#[napi]
pub fn parse_route(input: RouteInput) -> napi::Result<RouteDef> {
    let method = input.method.trim().to_ascii_uppercase();
    if !ALL_METHODS.contains(&method.as_str()) {
        return Err(NapiError::new(
            Status::InvalidArg,
            format!(
                "invalid HTTP method `{}`; expected one of: {}",
                input.method,
                ALL_METHODS.join(", ")
            ),
        ));
    }

    let pattern = parse_pattern(&input.path);
    let param_types = override_map(&input.param_types);
    let query_types = override_map(&input.query_types);

    let params = pattern
        .path_params
        .iter()
        .enumerate()
        .map(|(index, (name, trailing))| RouteParam {
            name: name.clone(),
            ts_name: sanitize_ident(name, index),
            kind: ParamKind::Path,
            // A multi-segment param (`*`, `<name..>`) is filled from an array.
            ts_type: param_types
                .get(name)
                .cloned()
                .unwrap_or_else(|| {
                    if *trailing {
                        "string[]".to_string()
                    } else {
                        "string | number".to_string()
                    }
                }),
            trailing: *trailing,
            optional: *trailing,
        })
        .collect();

    let query = pattern
        .query
        .iter()
        .map(|name| RouteParam {
            name: name.clone(),
            ts_name: name.clone(),
            kind: ParamKind::Query,
            ts_type: query_types
                .get(name)
                .cloned()
                .unwrap_or_else(|| "string | number".to_string()),
            trailing: false,
            optional: true,
        })
        .collect();

    let key = format!("{method} {}", pattern.ts_path);
    let name = input
        .name
        .clone()
        .filter(|n| !n.trim().is_empty())
        .map(|n| sanitize_ident(&n, 0))
        .unwrap_or_else(|| derive_method_name(&method, &pattern.ts_path));

    Ok(RouteDef {
        key,
        method,
        path: pattern.ts_path,
        rocket_path: pattern.rocket_path,
        name,
        params,
        query,
        body: input.body.clone(),
        response: input.response.clone(),
    })
}

/// Resolve a route table into `Vec<RouteDef>` with unique handler names.
#[napi]
pub fn parse_routes(inputs: Vec<RouteInput>) -> napi::Result<Vec<RouteDef>> {
    let mut defs: Vec<RouteDef> = inputs.into_iter().map(parse_route).collect::<napi::Result<_>>()?;
    dedupe_names(&mut defs);
    Ok(defs)
}

/// Translate an Alloy/Bun or Rocket pattern into Rocket's `<param>` syntax —
/// the same translation [`crate::server::serve`] applies before mounting.
#[napi]
pub fn to_rocket_path(pattern: String) -> String {
    parse_pattern(&pattern).rocket_path
}

// ---------------------------------------------------------------------------
// Emission
// ---------------------------------------------------------------------------

const BANNER: &str = "// Code generated by alloy-server-native (Rocket codegen). DO NOT EDIT.\n\n";

const REQUEST_LIKE_DECL: &str = r#"/**
 * Structural mirror of the `RequestInput` object the NAPI server passes to a
 * JS route handler (see `server.rs`). Declared locally so the generated file
 * needs no runtime imports; pass `serverImport` to use the real type instead.
 */
export interface RequestLike {
  method: string;
  path: string;
  query: string;
  headers: Array<{ name: string; value: string }>;
  body?: Uint8Array;
}
"#;

const URI_HELPER: &str = r#"export type QueryValue = string | number | boolean | null | undefined;

export interface UriParams {
  [name: string]: string | number | boolean | null | undefined | Array<string | number>;
}

const PARAM_PATTERN = /:([A-Za-z_][A-Za-z0-9_]*)/g;

/**
 * Build a URL for a route key (`"GET /users/:id"`), interpolating path params
 * and appending the query string — the runtime counterpart of Rocket's `uri!`.
 */
export function uri(key: string, params: UriParams = {}, query: Record<string, QueryValue> = {}): string {
  const template = key.slice(key.indexOf(" ") + 1);
  let path = template.replace(PARAM_PATTERN, (_match, name: string) => {
    const value = params[name];
    if (value === undefined || value === null) {
      throw new Error(`uri: missing param "${name}" for route "${key}"`);
    }
    return encodeURIComponent(String(value));
  });
  if (path.includes("*")) {
    const rest = params["_"];
    path = path.replace("*", Array.isArray(rest) ? rest.map((part) => encodeURIComponent(String(part))).join("/") : "");
  }
  const search = Object.entries(query)
    .filter(([, value]) => value !== undefined && value !== null && value !== "")
    .map(([name, value]) => `${encodeURIComponent(name)}=${encodeURIComponent(String(value))}`)
    .join("&");
  return search.length > 0 ? `${path}?${search}` : path;
}
"#;

const CLIENT_SEND: &str = r#"  private async send(key: string, params: UriParams, query: Record<string, QueryValue>, body: unknown, init?: RequestInit): Promise<Response> {
    const fetchFn = this.options.fetch ?? fetch;
    const headers: Record<string, string> = { ...this.options.headers, ...(init?.headers as Record<string, string> | undefined) };
    let payload: string | undefined;
    if (body !== undefined) {
      if (!("content-type" in headers)) headers["content-type"] = "application/json";
      payload = JSON.stringify(body);
    }
    return fetchFn(this.baseUrl + uri(key, params, query), { ...init, method: key.slice(0, key.indexOf(" ")), headers, body: payload });
  }
"#;

fn push_banner(out: &mut String, options: &CodegenOptions) {
    if options.banner.unwrap_or(true) {
        out.push_str(BANNER);
    }
}

fn push_request_like(out: &mut String, options: &CodegenOptions) {
    match &options.server_import {
        Some(module) => {
            let _ = writeln!(out, "import type {{ RequestInput as RequestLike }} from \"{module}\";");
            out.push('\n');
        }
        None => {
            out.push_str(REQUEST_LIKE_DECL);
            out.push('\n');
        }
    }
}

fn push_members(out: &mut String, params: &[RouteParam], indent: &str) {
    if params.is_empty() {
        out.push_str("Record<string, never>");
        return;
    }
    out.push_str("{\n");
    for param in params {
        let optional = if param.optional { "?" } else { "" };
        let _ = writeln!(out, "{indent}  {}{optional}: {};", param.ts_name, param.ts_type);
    }
    out.push_str(indent);
    out.push('}');
}

/// The route map interface plus the `ParamsOf` / `QueryOf` / … aliases and the
/// handler signatures that consume them.
fn push_route_types(out: &mut String, defs: &[RouteDef], options: &CodegenOptions) {
    let types_name = options
        .types_name
        .clone()
        .unwrap_or_else(|| "RouteTypes".to_string());

    let _ = writeln!(out, "export interface {types_name} {{");
    for def in defs {
        let _ = writeln!(out, "  \"{}\": {{", def.key);
        out.push_str("    params: ");
        push_members(out, &def.params, "    ");
        out.push_str(";\n    query: ");
        push_members(out, &def.query, "    ");
        out.push_str(";\n    body: ");
        out.push_str(def.body.as_deref().unwrap_or("Record<string, never>"));
        out.push_str(";\n    response: ");
        out.push_str(def.response.as_deref().unwrap_or("unknown"));
        out.push_str(";\n  };\n");
    }
    out.push_str("}\n\n");

    let _ = writeln!(out, "export type RouteKey = keyof {types_name};");
    let _ = writeln!(out, "export type ParamsOf<K extends RouteKey> = {types_name}[K][\"params\"];");
    let _ = writeln!(out, "export type QueryOf<K extends RouteKey> = {types_name}[K][\"query\"];");
    let _ = writeln!(out, "export type BodyOf<K extends RouteKey> = {types_name}[K][\"body\"];");
    let _ = writeln!(out, "export type ResponseOf<K extends RouteKey> = {types_name}[K][\"response\"];");
    out.push('\n');

    let _ = writeln!(out, "/** Context handed to a generated route handler; mirrors `RequestInput`. */");
    // `RequestInput.query` is the raw query *string*; the generated `query` is
    // the parsed object, so it must be omitted rather than overridden.
    let _ = writeln!(out, "export interface RouteContext<K extends RouteKey> extends Omit<RequestLike, \"query\"> {{");
    let _ = writeln!(out, "  params: ParamsOf<K>;");
    let _ = writeln!(out, "  query: QueryOf<K>;");
    let _ = writeln!(out, "}}");
    out.push('\n');
    let _ = writeln!(out, "/** Signature every route handler must satisfy. */");
    let _ = writeln!(out, "export type RouteHandler<K extends RouteKey> = (ctx: RouteContext<K>) => Response | Promise<Response>;");
    out.push('\n');
}

fn push_routes_const(out: &mut String, defs: &[RouteDef]) {
    let _ = writeln!(out, "export const ROUTES = {{");
    for def in defs {
        let _ = writeln!(out, "  \"{}\": \"{}\",", def.key, def.path);
    }
    let _ = writeln!(out, "}} as const;");
    out.push('\n');
}

fn push_client(out: &mut String, defs: &[RouteDef], options: &CodegenOptions) {
    let client = options
        .client_name
        .clone()
        .unwrap_or_else(|| "RocketClient".to_string());
    let default_base = options.base_url.clone().unwrap_or_default();

    out.push_str("export interface ");
    out.push_str(&client);
    out.push_str("Options {\n  baseUrl?: string;\n  fetch?: typeof fetch;\n  headers?: Record<string, string>;\n}\n\n");

    out.push_str("export class ");
    out.push_str(&client);
    out.push_str(" {\n  private readonly options: ");
    out.push_str(&client);
    out.push_str("Options;\n\n  constructor(options: ");
    out.push_str(&client);
    out.push_str("Options = {}) {\n    this.options = options;\n  }\n\n");
    out.push_str("  private get baseUrl(): string {\n    return this.options.baseUrl ?? \"");
    out.push_str(&default_base);
    out.push_str("\";\n  }\n\n");
    out.push_str(CLIENT_SEND);
    out.push('\n');

    for def in defs {
        let mut args = Vec::new();
        let mut send_args = vec![format!("\"{}\"", def.key)];
        if def.params.is_empty() {
            send_args.push("{}".to_string());
        } else {
            args.push(format!("params: ParamsOf<\"{}\">", def.key));
            send_args.push("params".to_string());
        }
        if def.query.is_empty() {
            send_args.push("{}".to_string());
        } else {
            args.push(format!("query?: QueryOf<\"{}\">", def.key));
            send_args.push("query ?? {}".to_string());
        }
        if def.body.is_some() {
            args.push(format!("body: BodyOf<\"{}\">", def.key));
            send_args.push("body".to_string());
        } else {
            send_args.push("undefined".to_string());
        }
        args.push("init?: RequestInit".to_string());
        send_args.push("init".to_string());

        let _ = writeln!(out, "  /** {} */", def.key);
        let _ = writeln!(
            out,
            "  async {}({}): Promise<ResponseOf<\"{}\">> {{",
            def.name,
            args.join(", "),
            def.key
        );
        let _ = writeln!(out, "    const response = await this.send({});", send_args.join(", "));
        let _ = writeln!(
            out,
            "    return (await response.json()) as ResponseOf<\"{}\">;",
            def.key
        );
        let _ = writeln!(out, "  }}");
        out.push('\n');
    }

    out.push_str("}\n");
}

/// Emit a `.d.ts`-style route map: params, query, body, and response types per
/// route, plus the `ParamsOf` / `QueryOf` / `BodyOf` / `ResponseOf` aliases and
/// the handler signature they feed.
#[napi]
pub fn generate_types(defs: Vec<RouteDef>, options: Option<CodegenOptions>) -> String {
    let options = options.unwrap_or_default();
    let mut defs = defs;
    dedupe_names(&mut defs);

    let mut out = String::new();
    push_banner(&mut out, &options);
    push_request_like(&mut out, &options);
    push_route_types(&mut out, &defs, &options);
    out
}

/// Emit a typed `fetch` client: one method per route, with the `uri()` builder
/// (the runtime counterpart of Rocket's `uri!`) alongside it.
#[napi]
pub fn generate_client(defs: Vec<RouteDef>, options: Option<CodegenOptions>) -> String {
    let options = options.unwrap_or_default();
    let mut defs = defs;
    dedupe_names(&mut defs);

    let mut out = String::new();
    push_banner(&mut out, &options);
    match &options.types_import {
        Some(module) => {
            let _ = writeln!(
                out,
                "import type {{ ParamsOf, QueryOf, BodyOf, ResponseOf }} from \"{module}\";"
            );
            out.push('\n');
        }
        None => {
            // Inline the types so the output stands on its own.
            push_request_like(&mut out, &options);
            push_route_types(&mut out, &defs, &options);
        }
    }
    out.push_str(URI_HELPER);
    out.push('\n');
    push_client(&mut out, &defs, &options);
    out
}

/// Emit a complete module: the `ROUTES` table, the route map types, the `uri()`
/// builder, and the typed client.
#[napi]
pub fn generate_module(defs: Vec<RouteDef>, options: Option<CodegenOptions>) -> String {
    let options = options.unwrap_or_default();
    let mut defs = defs;
    dedupe_names(&mut defs);

    let mut out = String::new();
    push_banner(&mut out, &options);
    push_request_like(&mut out, &options);
    push_routes_const(&mut out, &defs);
    push_route_types(&mut out, &defs, &options);
    out.push_str(URI_HELPER);
    out.push('\n');
    push_client(&mut out, &defs, &options);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(method: &str, path: &str) -> RouteInput {
        RouteInput {
            method: method.to_string(),
            path: path.to_string(),
            name: None,
            body: None,
            response: None,
            param_types: None,
            query_types: None,
        }
    }

    #[test]
    fn parses_bun_style_path() {
        let def = parse_route(input("get", "/users/:id/posts/*")).unwrap();
        assert_eq!(def.method, "GET");
        assert_eq!(def.path, "/users/:id/posts/*");
        assert_eq!(def.rocket_path, "/users/<id>/posts/<_..>");
        assert_eq!(def.key, "GET /users/:id/posts/*");
        assert_eq!(def.name, "getUsersByIdPosts");
        assert_eq!(def.params.len(), 2);
        assert_eq!(def.params[0].name, "id");
        assert!(!def.params[0].trailing);
        assert!(!def.params[0].optional);
        assert_eq!(def.params[1].name, "_");
        assert!(def.params[1].trailing);
        assert!(def.params[1].optional);
    }

    #[test]
    fn parses_rocket_style_path_and_query() {
        let def = parse_route(input("POST", "/users/<id..>?<q>&closed")).unwrap();
        assert_eq!(def.path, "/users/:id");
        assert_eq!(def.rocket_path, "/users/<id..>");
        assert_eq!(def.key, "POST /users/:id");
        assert!(def.params[0].trailing);
        assert_eq!(def.query.len(), 2);
        assert_eq!(def.query[0].name, "q");
        assert_eq!(def.query[1].name, "closed");
        assert!(def.query.iter().all(|q| q.optional));
    }

    #[test]
    fn root_and_unknown_methods() {
        assert_eq!(parse_route(input("GET", "/")).unwrap().name, "get");
        assert_eq!(to_rocket_path("/".to_string()), "/");
        assert!(parse_route(input("FETCH", "/")).is_err());
    }

    #[test]
    fn lists_http_methods() {
        let methods = http_methods();
        assert_eq!(methods.len(), ALL_METHODS.len());
        assert!(methods.contains(&"GET".to_string()));
        assert!(methods.contains(&"PATCH".to_string()));
        assert!(!methods.contains(&"FETCH".to_string()));
    }

    #[test]
    fn sanitizes_reserved_and_invalid_params() {
        let mut route = input("GET", "/items/:type?2fa");
        route.param_types = Some(vec![TypeOverride {
            name: "type".to_string(),
            ts_type: "\"a\" | \"b\"".to_string(),
        }]);
        let def = parse_route(route).unwrap();
        // `type` is reserved in TypeScript, so it is emitted as `type_`.
        assert_eq!(def.params[0].ts_name, "type_");
        assert_eq!(def.params[0].ts_type, "\"a\" | \"b\"");
        // A query name that is not a valid identifier is sanitized, not dropped.
        assert_eq!(def.query[0].ts_name, "param0_2fa");
        assert_eq!(def.query[0].name, "param0_2fa");

        // A path segment that cannot be a parameter stays literal.
        let def = parse_route(input("GET", "/items/:2fa")).unwrap();
        assert!(def.params.is_empty());
        assert_eq!(def.path, "/items/:2fa");
    }

    #[test]
    fn dedupes_route_names() {
        // The same pattern registered twice collapses to one generated name.
        let defs = parse_routes(vec![input("GET", "/a/:id"), input("GET", "/a/:id")]).unwrap();
        assert_eq!(defs[0].name, "getAById");
        assert_eq!(defs[1].name, "getAById_2");
    }

    #[test]
    fn translates_paths() {
        assert_eq!(to_rocket_path("/a/:b/*".to_string()), "/a/<b>/<_..>");
        assert_eq!(to_rocket_path("/a/<b..>".to_string()), "/a/<b..>");
        assert_eq!(to_rocket_path("/a/<_>".to_string()), "/a/<_..>");
    }

    #[test]
    fn emits_typescript() {
        let mut route = input("GET", "/users/:id?<active>");
        route.response = Some("User".to_string());
        let def = parse_route(route).unwrap();

        let ts = generate_module(vec![def.clone()], None);
        assert!(ts.contains("export interface RouteTypes {"));
        assert!(ts.contains("\"GET /users/:id\": {"));
        assert!(ts.contains("id: string | number;"));
        assert!(ts.contains("active?: string | number;"));
        assert!(ts.contains("response: User;"));
        assert!(ts.contains("export const ROUTES = {"));
        assert!(ts.contains("export function uri("));
        assert!(ts.contains("async getUsersById("));
        assert!(ts.contains("as ResponseOf<\"GET /users/:id\">;"));

        // `generate_types` emits declarations only: no values, no client.
        let types = generate_types(vec![def.clone()], None);
        assert!(types.contains("export interface RouteTypes {"));
        assert!(types.contains("export type ParamsOf<K extends RouteKey> = RouteTypes[K][\"params\"];"));
        assert!(types.contains("export interface RouteContext<K extends RouteKey> extends Omit<RequestLike, \"query\"> {"));
        assert!(types.contains("export type RouteHandler<K extends RouteKey> ="));
        assert!(types.contains("export interface RequestLike {"));
        assert!(!types.contains("export class"));
        assert!(!types.contains("export function uri("));

        let mut route = input("GET", "/users/:id?<active>");
        route.response = Some("User".to_string());
        let client = generate_client(vec![parse_route(route).unwrap()], None);
        assert!(client.contains("export class RocketClient {"));
        assert!(!client.contains("import type"));

        let client = generate_client(
            vec![def],
            Some(CodegenOptions {
                types_import: Some("./api-types".to_string()),
                client_name: Some("Api".to_string()),
                base_url: Some("http://localhost:3000".to_string()),
                ..Default::default()
            }),
        );
        assert!(client.contains("import type { ParamsOf, QueryOf, BodyOf, ResponseOf } from \"./api-types\";"));
        assert!(client.contains("export class Api {"));
        assert!(client.contains("http://localhost:3000"));
    }
}

//! # Rocket - Code Generation
//!
//! This module implements the code generation portions of Rocket and its
//! NAPI bindings for TypeScript (Node.js). This includes custom derives,
//! custom attributes, procedural macros, and runtime code-generation bridges.
//!
//! # Usage
//!
//! In Rust, Rocket uses procedural macros (`#[get]`, `#[post]`, `routes!`, `uri!`,
//! `#[derive(Responder)]`, etc.) to generate handlers and type checking at compile-time.
//!
//! In TypeScript (Node.js), procedural macros cannot cross the N-API boundary directly.
//! This module provides the NAPI bindings and helpers that mirror Rocket's code-generation
//! capabilities at runtime and build-time for Node.js/TypeScript applications:
//!
//! * **Route Generation**: `routes!` / `#[route]` -> `serve({ routes })` or `register_route`.
//! * **Catcher Generation**: `catchers!` / `#[catch]` -> `Rocket::register`.
//! * **URI Generation**: `uri!` -> `uri(pattern, params, query)`.
//! * **Response & Guard Generation**: `Responder`, `FromForm`, `FromData` -> `Response`, `responseJson`, `RequestInput`.

use std::collections::HashMap;
use napi::bindgen_prelude::*;
use napi_derive::napi;

/// NAPI helper function providing the functionality of Rocket's `uri!` macro
/// for TypeScript / Node.js.
///
/// Constructs a URL-safe URI string given a Rocket route pattern, path parameters,
/// and query parameters.
#[napi]
pub fn uri(
    pattern: String,
    params: Option<HashMap<String, String>>,
    query: Option<HashMap<String, String>>,
) -> Result<String> {
    let mut result = pattern;
    if let Some(params) = params {
        for (k, v) in params {
            let encoded = percent_encoding(&v);
            result = result.replace(&format!("<{}>", k), &encoded);
            result = result.replace(&format!("<{}..>", k), &encoded);
            result = result.replace(&format!(":{}", k), &encoded);
        }
    }
    if let Some(query) = query {
        if !query.is_empty() {
            let mut q_pairs = Vec::new();
            for (k, v) in query {
                let encoded_v = percent_encoding(&v);
                q_pairs.push(format!("{}={}", k, encoded_v));
            }
            let q_str = q_pairs.join("&");
            if result.contains('?') {
                result = format!("{}&{}", result, q_str);
            } else {
                result = format!("{}?{}", result, q_str);
            }
        }
    }
    Ok(result)
}

fn percent_encoding(s: &str) -> String {
    let mut encoded = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}

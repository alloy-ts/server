//! Rocket Code Generation NAPI Bridge
//!
//! This module documents and exports the Rust codegen bridge for Rocket in Node.js / TypeScript.

use napi_derive::napi;
use std::collections::HashMap;

/// Type-safe URI builder helper, replacing Rocket's `uri!` macro.
#[napi]
pub fn uri(path: String, params: Option<HashMap<String, String>>) -> String {
    let mut result = path;
    if let Some(params) = params {
        for (key, val) in params {
            let p1 = format!(":{key}");
            let p2 = format!("<{key}>");
            result = result.replace(&p1, &val).replace(&p2, &val);
        }
    }
    result
}

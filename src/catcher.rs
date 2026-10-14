//! # Rocket - Catcher NAPI Bindings
//!
//! This module implements NAPI bindings for Rocket's [`Catcher`] (error-catching routes).
//!
//! Catchers are routes that execute when an error or forward occurs in the application
//! (e.g. 404 Not Found, 500 Internal Server Error, 422 Unprocessable Entity, or custom status codes).
//!
//! Rocket provides built-in default catchers that produce HTML or JSON depending on the `Accept` header.

use napi_derive::napi;
use std::collections::HashMap;

use crate::server::Response;

/// Metadata representation of a Rocket Catcher.
#[napi(object)]
#[derive(Debug, Clone)]
pub struct CatcherInfo {
    /// The optional name of this catcher.
    pub name: Option<String>,
    /// The mount point base URI path (default "/").
    pub base: String,
    /// The HTTP status code to match against (400..600), or `None` for default catchers.
    pub code: Option<u16>,
}

/// Construct a `CatcherInfo` object.
///
/// Validates that `code` is in the valid HTTP error status range `[400, 600)` if specified.
#[napi]
pub fn create_catcher_info(
    code: Option<u16>,
    base: Option<String>,
    name: Option<String>,
) -> napi::Result<CatcherInfo> {
    if let Some(c) = code {
        if !(400..600).contains(&c) {
            return Err(napi::Error::new(
                napi::Status::InvalidArg,
                format!("status code {c} is outside valid error range [400, 600)"),
            ));
        }
    }

    Ok(CatcherInfo {
        name,
        base: base.unwrap_or_else(|| "/".to_string()),
        code,
    })
}

/// Helper generating Rocket's default HTML/JSON error response payload for a status code.
#[napi]
pub fn default_catcher_response(code: u32, is_json: Option<bool>) -> Response {
    let is_json = is_json.unwrap_or(false);
    let mut headers = HashMap::new();

    let text = if is_json {
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        format!(
            "{{\"error\": {{\"code\": {}, \"reason\": \"HTTP Error {}\"}}}}",
            code, code
        )
    } else {
        headers.insert(
            "Content-Type".to_string(),
            "text/html; charset=utf-8".to_string(),
        );
        format!(
            "<!DOCTYPE html><html><head><title>{} Error</title></head><body><h1>{} Error</h1></body></html>",
            code, code
        )
    };

    Response {
        status: code,
        headers,
        body: Some(text.into_bytes()),
    }
}

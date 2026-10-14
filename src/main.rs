//! NAPI exports for Rocket / Alloy web framework.

use napi_derive::napi;
use crate::server::{Response, ResponseInit};

#[napi]
pub fn main() -> String {
    "Hello, world!".to_string()
}

/// Helper function `response(body?, init?)` mirroring Rocket's response constructor.
#[napi]
pub fn response(body: Option<String>, init: Option<ResponseInit>) -> Response {
    Response::new(body, init)
}

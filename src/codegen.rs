//! # Rocket - Code Generation
//!
//! This module documents and provides bindings for the code generation portions of Rocket.
//! Custom derives, custom attributes, and procedural macros are supported.

use napi_derive::napi;

/// Documentation summary for Rocket code generation.
pub const CODEGEN_DOC: &str = r#"
Rocket Code Generation Facilities
---------------------------------
Attributes:
  - #[get], #[put], #[post], #[delete], #[head], #[options], #[patch]
  - #[route(METHOD, uri = "...")]
  - #[catch(code)]
  - #[launch]
  - #[rocket::main]
  - #[async_test]

Derives:
  - #[derive(FromFormField)]
  - #[derive(FromForm)]
  - #[derive(Responder)]
  - #[derive(UriDisplayQuery)]
  - #[derive(UriDisplayPath)]

Macros:
  - routes![...]
  - catchers![...]
  - uri!(...)
"#;

/// Returns the documentation summary for Rocket code generation.
#[napi]
pub fn get_codegen_doc() -> String {
    CODEGEN_DOC.to_string()
}

/// Returns true if ROCKET_CODEGEN_DEBUG environment variable is set.
#[napi]
pub fn is_codegen_debug_enabled() -> bool {
    std::env::var_os("ROCKET_CODEGEN_DEBUG").is_some()
}

/// Helper function returning Rocket Code Generation module info.
#[napi]
pub fn get_codegen_info() -> String {
    "Rocket Code Generation Module".to_string()
}

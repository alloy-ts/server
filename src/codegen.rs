//! # Rocket - Code Generation
//!
//! This crate/module documents and bridges the code generation facilities for Rocket
//! web framework in Node.js and TypeScript.
//!
//! In Rust, Rocket relies on procedural macros (`#[get]`, `routes!`, `derive(...)`,
//! `uri!`) provided by `rocket_codegen`. In the TypeScript/NAPI layer, these
//! responsibilities are supported via TypeScript decorators (`@Get`, `@Post`,
//! `@Put`, `@Delete`, `@Patch`, `@Head`, `@Options`, `@Route`, `@Catch`) and
//! runtime/type-safe helpers like `uri()`.

/// Returns metadata about the Rocket codegen bridge.
pub fn codegen_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

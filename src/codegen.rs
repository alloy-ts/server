//! Rocket Code Generation NAPI Bridge
//!
//! This module documents and exports the Rust codegen bridge for Rocket in Node.js / TypeScript.
//! Rocket's procedural macros (e.g. `#[get]`, `#[post]`, `routes!`, `uri!`) generate route
//! metadata and wrappers in Rust at compile time.
//!
//! For TypeScript / Node.js, this codegen layer translates Rocket macro responsibilities
//! into TypeScript decorators (`@get`, `@post`, `@put`, `@del`), `registerRoute`, and `uri`
//! parameter interpolation helpers.

pub use crate::server::*;

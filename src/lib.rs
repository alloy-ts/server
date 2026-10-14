#![recursion_limit = "256"]

//! # Rocket - NAPI Bindings & Core API Documentation
//!
//! Hello, and welcome to the core Rocket NAPI API documentation!
//!
//! This crate provides NAPI-RS bindings for the Rocket web framework and code
//! generation utilities to drive high-performance HTTP servers from Node.js and
//! TypeScript.

mod codegen;
mod config;
#[allow(special_module_name)]
mod main;
pub mod route;
mod server;

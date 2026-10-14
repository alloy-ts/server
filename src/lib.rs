#![recursion_limit = "256"]
#![doc(html_root_url = "https://api.rocket.rs/v0.5")]
#![doc(html_favicon_url = "https://rocket.rs/images/favicon.ico")]
#![doc(html_logo_url = "https://rocket.rs/images/logo-boxed.png")]

//! # Rocket - NAPI Bindings & Core API
//!
//! NAPI bindings that expose the Rocket web framework to Node.js and TypeScript.

mod codegen;
mod config;
mod handler;
mod rocket_app;
mod route;
mod route_uri;
mod server;

pub use codegen::*;
pub use config::*;
pub use handler::*;
pub use rocket_app::{rocket_main, Rocket, RocketConfig};
pub use route::*;
pub use route_uri::*;
pub use server::*;

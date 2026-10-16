use napi_derive::napi;

pub mod codegen;
pub mod config;
pub mod handler;
pub mod rocket_main;
pub mod route;
pub mod route_uri;
pub mod server;

pub use codegen::*;
pub use config::*;
pub use handler::*;
pub use rocket_main::*;
pub use route::*;
pub use route_uri::*;
pub use server::*;

#[napi]
pub fn add(left: i32, right: i32) -> i32 {
    left + right
}

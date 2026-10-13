use napi_derive::napi;

pub mod codegen;
pub mod config;
pub mod rocket_main;
pub mod server;

pub use codegen::*;
pub use config::*;
pub use rocket_main::*;
pub use server::*;

#[napi]
pub fn add(left: i32, right: i32) -> i32 {
    left + right
}

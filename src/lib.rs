use napi_derive::napi;

mod main;
mod server;

#[napi]
pub fn add(left: i32, right: i32) -> i32 {
  left + right
}

//! NAPI bindings for Rocket's [`Handler`] trait (`rocket::route::Handler`).
//!
//! Rocket's `Handler` trait processes incoming HTTP requests (`Request`) and
//! request data (`Data`) to produce an `Outcome` (Success response, Error status,
//! or Forward to the next route).
//!
//! In TypeScript / Node.js, JS handler functions (`(req: RequestInput) => Promise<ResponsePayload>`)
//! are bridged to Rocket's `Handler` trait via thread-safe callbacks (`ThreadsafeFunction`).

pub use crate::server::*;

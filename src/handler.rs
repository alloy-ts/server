//! # Rocket - Handler NAPI Bindings
//!
//! This module defines the NAPI bindings for Rocket's [`Handler`] trait.
//!
//! In Rocket, request handlers implement the [`Handler`] trait, which processes an incoming
//! [`Request`] and [`Data`] stream to produce a route [`Outcome`].
//!
//! Rocket's procedural macros (`#[get]`, `#[post]`, etc.) generate implementations of `Handler`
//! automatically in Rust. For TypeScript/Node.js, this crate wraps JavaScript/TypeScript functions
//! and custom handler callbacks into Rocket-compatible `Handler` implementations via Threadsafe Functions.

use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

use napi_derive::napi;

use rocket::data::Data;
use rocket::http::Status as RocketStatus;
use rocket::request::Request;
use rocket::response::Response as RocketResponse;
use rocket::route::{Handler, Outcome as RouteOutcome};

use crate::server::{build_request, build_response, RouteKind};

/// Representation of a Rocket route handling outcome for JavaScript/TypeScript handlers.
///
/// Corresponds to Rocket's `rocket::route::Outcome`:
/// - `Success`: The request was successfully handled, returning a response.
/// - `Error`: The request resulted in an HTTP error status (invoking Rocket's error catchers).
/// - `Forward`: The request should be forwarded to the next matching route.
#[napi(string_enum)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum OutcomeKind {
    Success,
    Error,
    Forward,
}

/// Handler outcome response object returned by or passed to JavaScript route handlers.
#[napi(object)]
pub struct HandlerOutcome {
    /// The outcome kind: "Success", "Error", or "Forward".
    pub kind: OutcomeKind,
    /// HTTP status code associated with Error or Forward outcomes.
    pub status: Option<u32>,
    /// Optional response body for Success outcomes.
    pub body: Option<Vec<u8>>,
    /// Optional headers for Success outcomes.
    pub headers: Option<HashMap<String, String>>,
}

impl HandlerOutcome {
    /// Create a successful outcome with a response body.
    pub fn success(body: Vec<u8>, headers: Option<HashMap<String, String>>) -> Self {
        Self {
            kind: OutcomeKind::Success,
            status: Some(200),
            body: Some(body),
            headers,
        }
    }

    /// Create an error outcome with a status code.
    pub fn error(status: u32) -> Self {
        Self {
            kind: OutcomeKind::Error,
            status: Some(status),
            body: None,
            headers: None,
        }
    }

    /// Create a forward outcome.
    pub fn forward(status: Option<u32>) -> Self {
        Self {
            kind: OutcomeKind::Forward,
            status,
            body: None,
            headers: None,
        }
    }
}

/// Rocket [`Handler`] implementation that delegates request processing to JavaScript via NAPI ThreadsafeFunction.
#[derive(Clone)]
pub struct JsHandler(pub(crate) Arc<RouteKind>);

#[rocket::async_trait]
impl Handler for JsHandler {
    async fn handle<'r>(&self, request: &'r Request<'_>, data: Data<'r>) -> RouteOutcome<'r> {
        let pattern = request
            .route()
            .map(|r| r.uri.to_string())
            .unwrap_or_default();
        match &*self.0 {
            RouteKind::Fn(tsfn) => {
                let input = build_request(request, data, &pattern).await;
                let output = match tsfn.call_async(Ok(input)).await {
                    Ok(promise) => match promise.await {
                        Ok(payload) => payload,
                        Err(_) => return RouteOutcome::Error(RocketStatus::InternalServerError),
                    },
                    Err(_) => return RouteOutcome::Error(RocketStatus::InternalServerError),
                };
                RouteOutcome::Success(build_response(&output))
            }
            RouteKind::Static(payload) => RouteOutcome::Success(build_response(payload.as_ref())),
            RouteKind::File { path } => match rocket::tokio::fs::read(path).await {
                Ok(bytes) => {
                    let mut builder = RocketResponse::build();
                    builder.sized_body(bytes.len(), Cursor::new(bytes));
                    RouteOutcome::Success(builder.finalize())
                }
                Err(_) => RouteOutcome::Error(RocketStatus::NotFound),
            },
        }
    }
}

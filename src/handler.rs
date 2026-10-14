//! NAPI bindings for Rocket's [`Handler`] trait (`rocket::route::Handler`).
//!
//! Bridges JS custom route handlers into Rocket's `Handler` trait, allowing
//! Node.js / TypeScript callers to return `outcomeFrom()`, `outcomeError()`,
//! or `outcomeForward()`.

use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
#[allow(deprecated)]
use napi::{JsFunction, Unknown};
use napi_derive::napi;

use rocket::data::ByteUnit;
use rocket::http::{Header as RocketHeader, Status as RocketStatus};
use rocket::request::Request;
use rocket::response::Response as RocketResponse;
use rocket::route::{Handler as RocketHandler, Outcome as RouteOutcome};
use rocket::Data;

use crate::server::{RequestInput, Response};

/// Representation of Rocket's `Outcome<Response, Status, (Data, Status)>`.
#[napi(object)]
#[derive(Clone, Debug)]
pub struct Outcome {
    /// `"success"`, `"error"`, or `"forward"`.
    pub kind: String,
    pub status: Option<u32>,
    pub response: Option<Response>,
}

/// Returns a `Success(Response)` outcome wrapping the responder payload.
#[napi]
pub fn outcome_from(response: Response) -> Outcome {
    Outcome {
        kind: "success".to_string(),
        status: Some(response.status),
        response: Some(response),
    }
}

/// Returns an `Error(Status)` outcome with HTTP status code.
#[napi]
pub fn outcome_error(status: u32) -> Outcome {
    Outcome {
        kind: "error".to_string(),
        status: Some(status),
        response: None,
    }
}

/// Returns a `Forward((Data, Status))` outcome to attempt the next matching route.
#[napi]
pub fn outcome_forward(status: Option<u32>) -> Outcome {
    Outcome {
        kind: "forward".to_string(),
        status: Some(status.unwrap_or(404)),
        response: None,
    }
}

type HandlerTsfn =
    ThreadsafeFunction<RequestInput, Promise<Response>, RequestInput, Status, false, false, 0>;

#[allow(deprecated)]
fn to_js_function(handler: Unknown) -> JsFunction {
    debug_assert_eq!(
        std::mem::size_of::<Unknown>(),
        std::mem::size_of::<JsFunction>()
    );
    unsafe { std::mem::transmute_copy::<Unknown, JsFunction>(&handler) }
}

/// NAPI wrapper around Rocket's `Handler` trait.
#[napi]
#[derive(Clone)]
pub struct Handler {
    tsfn: Arc<HandlerTsfn>,
}

#[napi]
impl Handler {
    /// Creates a custom Rocket `Handler` from a JavaScript callback function.
    #[napi(constructor)]
    #[allow(deprecated)]
    pub fn new(handler: Unknown) -> napi::Result<Self> {
        let f = to_js_function(handler);
        let tsfn: HandlerTsfn =
            f.create_threadsafe_function::<RequestInput, RequestInput, Promise<Response>, Status, _, false, false, 0>(
                |ctx| Ok(ctx.value),
            )?;
        Ok(Self {
            tsfn: Arc::new(tsfn),
        })
    }
}

#[rocket::async_trait]
impl RocketHandler for Handler {
    async fn handle<'r>(&self, request: &'r Request<'_>, data: Data<'r>) -> RouteOutcome<'r> {
        let pattern = request
            .route()
            .map(|r| r.uri.to_string())
            .unwrap_or_default();
        let input = build_request_input(request, data, &pattern).await;

        match self.tsfn.call_async(input).await {
            Ok(promise) => match promise.await {
                Ok(resp) => RouteOutcome::Success(build_rocket_response(&resp)),
                Err(_) => RouteOutcome::Error(RocketStatus::InternalServerError),
            },
            Err(_) => RouteOutcome::Error(RocketStatus::InternalServerError),
        }
    }
}

async fn build_request_input(
    request: &Request<'_>,
    data: Data<'_>,
    pattern: &str,
) -> RequestInput {
    let method = request.method().as_str().to_string();
    let uri = request.uri();
    let path = uri.path().to_string();
    let query = uri.query().map(|q| q.to_string()).unwrap_or_default();
    let headers = request
        .headers()
        .iter()
        .map(|h| (h.name.as_str().to_string(), h.value().to_string()))
        .collect();

    let body = match data.open(ByteUnit::Megabyte(1)).into_bytes().await {
        Ok(capped) => Some(capped.value),
        Err(_) => None,
    };

    let params = extract_params(pattern, &path);

    RequestInput {
        method,
        path,
        query,
        headers,
        body,
        params,
    }
}

fn extract_params(pattern: &str, path: &str) -> HashMap<String, String> {
    let p_segs: Vec<&str> = pattern
        .trim_start_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();
    let a_segs: Vec<&str> = path
        .trim_start_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();
    let mut params = HashMap::new();
    for (idx, seg) in p_segs.iter().enumerate() {
        if let Some(inner) = seg.strip_prefix('<') {
            let name = inner
                .strip_suffix("..>")
                .or_else(|| inner.strip_suffix('>'));
            if let Some(name) = name {
                if name != "_" {
                    if let Some(v) = a_segs.get(idx) {
                        params.insert(name.to_string(), (*v).to_string());
                    }
                }
            }
        }
    }
    params
}

fn build_rocket_response(payload: &Response) -> RocketResponse<'static> {
    let mut builder = RocketResponse::build();
    if let Some(status) = RocketStatus::from_code(payload.status as u16) {
        builder.status(status);
    }
    for (name, value) in &payload.headers {
        builder.header(RocketHeader::new(name.clone(), value.clone()));
    }
    if let Some(body) = &payload.body {
        let bytes = body.clone();
        builder.sized_body(bytes.len(), Cursor::new(bytes));
    }
    builder.finalize()
}

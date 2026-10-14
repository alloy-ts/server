//! # Rocket Handler NAPI Bindings
//!
//! Exposes Rocket's `Handler` trait and `Outcome` type to TypeScript, allowing
//! custom request handlers and outcome decisions (`Success`, `Error`, `Forward`).

use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
#[allow(deprecated)]
use napi::{JsFunction, Unknown};
use napi_derive::napi;

use rocket::http::Status as RocketStatus;
use rocket::request::Request;
use rocket::route::{Handler as RocketHandler, Outcome as RocketOutcome};
use rocket::Data;

use crate::server::{extract_params, RequestInput, Response};

#[allow(deprecated)]
fn to_js_function(handler: Unknown) -> JsFunction {
    debug_assert_eq!(
        std::mem::size_of::<Unknown>(),
        std::mem::size_of::<JsFunction>()
    );
    unsafe { std::mem::transmute_copy::<Unknown, JsFunction>(&handler) }
}

/// NAPI representation of Rocket's request handling outcome.
#[napi(object)]
pub struct Outcome {
    /// `"success"`, `"error"`, or `"forward"`.
    pub status: String,
    pub response: Option<Response>,
    pub code: Option<u32>,
}

/// Create a `Success` outcome wrapping a response.
#[napi(js_name = "outcomeSuccess")]
pub fn outcome_success(response: Response) -> Outcome {
    Outcome {
        status: "success".to_string(),
        response: Some(response),
        code: None,
    }
}

/// Create an `Error` outcome with an HTTP status code.
#[napi(js_name = "outcomeError")]
pub fn outcome_error(code: u32) -> Outcome {
    Outcome {
        status: "error".to_string(),
        response: None,
        code: Some(code),
    }
}

/// Create a `Forward` outcome forwarding the request to the next route.
#[napi(js_name = "outcomeForward")]
pub fn outcome_forward(code: Option<u32>) -> Outcome {
    Outcome {
        status: "forward".to_string(),
        response: None,
        code,
    }
}

/// NAPI wrapper around a JS callback implementing Rocket's `Handler` trait.
#[napi]
#[derive(Clone)]
pub struct Handler {
    tsfn: Arc<ThreadsafeFunction<RequestInput, Promise<Outcome>, RequestInput, Status, false, false, 0>>,
}

#[napi]
impl Handler {
    #[napi(constructor)]
    #[allow(deprecated)]
    pub fn new(callback: Unknown) -> napi::Result<Self> {
        let f = to_js_function(callback);
        let tsfn = f.create_threadsafe_function::<RequestInput, RequestInput, Promise<Outcome>, Status, _, false, false, 0>(
            |ctx| Ok(ctx.value),
        )?;
        Ok(Self {
            tsfn: Arc::new(tsfn),
        })
    }
}

#[rocket::async_trait]
impl RocketHandler for Handler {
    async fn handle<'r>(&self, request: &'r Request<'_>, data: Data<'r>) -> RocketOutcome<'r> {
        let pattern = request
            .route()
            .map(|r| r.uri.to_string())
            .unwrap_or_default();
        let method = request.method().as_str().to_string();
        let uri = request.uri();
        let path = uri.path().to_string();
        let query = uri.query().map(|q| q.to_string()).unwrap_or_default();
        let headers = request
            .headers()
            .iter()
            .map(|h| (h.name.as_str().to_string(), h.value().to_string()))
            .collect();

        let params = extract_params(&pattern, &path);

        let input = RequestInput {
            method,
            path,
            query,
            headers,
            body: None,
            params,
        };

        match self.tsfn.call_async(input).await {
            Ok(promise) => match promise.await {
                Ok(outcome) => match outcome.status.as_str() {
                    "success" => {
                        if let Some(resp) = outcome.response {
                            RocketOutcome::Success(crate::server::build_response(&resp))
                        } else {
                            RocketOutcome::Error(RocketStatus::InternalServerError)
                        }
                    }
                    "error" => {
                        let code = outcome.code.unwrap_or(500);
                        let status = RocketStatus::from_code(code as u16)
                            .unwrap_or(RocketStatus::InternalServerError);
                        RocketOutcome::Error(status)
                    }
                    "forward" => {
                        let code = outcome.code.unwrap_or(404);
                        let status = RocketStatus::from_code(code as u16)
                            .unwrap_or(RocketStatus::NotFound);
                        RocketOutcome::Forward((data, status))
                    }
                    _ => RocketOutcome::Error(RocketStatus::InternalServerError),
                },
                Err(_) => RocketOutcome::Error(RocketStatus::InternalServerError),
            },
            Err(_) => RocketOutcome::Error(RocketStatus::InternalServerError),
        }
    }
}

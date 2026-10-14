//! NAPI bindings for Rocket's [`Handler`](rocket::route::Handler) trait.
//!
//! Exposes custom route handlers that can process requests, read input payloads,
//! and return `Outcome::Success(Response)`, `Outcome::Error(Status)`, or `Outcome::Forward(Data)`.

use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
use napi::Unknown;
use napi_derive::napi;

use rocket::http::Status as RocketStatus;
use rocket::request::Request;
use rocket::route::{Handler as RocketRouteHandler, Outcome as RouteOutcome};
use rocket::Data;

use crate::rocket_app::{build_request_payload, build_response, MainResponsePayload, RequestPayload};

/// Outcome type returned by a custom JS handler:
/// `type`: `"success"` | `"error"` | `"forward"`
#[napi(object)]
pub struct HandlerOutcome {
    pub kind: String,
    pub response: Option<MainResponsePayload>,
    pub status: Option<u32>,
}

type CustomHandlerTsfn = ThreadsafeFunction<
    RequestPayload,
    Either<HandlerOutcome, Promise<HandlerOutcome>>,
    (RequestPayload,),
    Status,
    false,
    false,
    0,
>;

#[allow(deprecated)]
fn to_js_function(handler: Unknown<'_>) -> napi::JsFunction {
    unsafe { std::mem::transmute_copy::<Unknown, napi::JsFunction>(&handler) }
}

/// Bridge wrapping a JS function callback that implements Rocket's [`Handler`](rocket::route::Handler) trait.
#[derive(Clone)]
#[napi]
pub struct Handler {
    tsfn: std::sync::Arc<CustomHandlerTsfn>,
}

#[napi]
impl Handler {
    /// Create a new custom request `Handler` wrapping a JavaScript async callback `(req) => outcome`.
    #[napi(constructor)]
    pub fn new(callback: Unknown<'_>) -> napi::Result<Self> {
        let tsfn: CustomHandlerTsfn = to_js_function(callback).create_threadsafe_function::<
            RequestPayload,
            (RequestPayload,),
            Either<HandlerOutcome, Promise<HandlerOutcome>>,
            Status,
            _,
            false,
            false,
            0,
        >(|ctx| Ok((ctx.value,)))?;

        Ok(Handler {
            tsfn: std::sync::Arc::new(tsfn),
        })
    }

    /// Helper creating a `Success` outcome with a given response payload.
    #[napi]
    pub fn success(response: MainResponsePayload) -> HandlerOutcome {
        HandlerOutcome {
            kind: "success".to_string(),
            response: Some(response),
            status: None,
        }
    }

    /// Helper creating an `Error` outcome with a given HTTP status code.
    #[napi]
    pub fn error(status: u32) -> HandlerOutcome {
        HandlerOutcome {
            kind: "error".to_string(),
            response: None,
            status: Some(status),
        }
    }

    /// Helper creating a `Forward` outcome to forward the request to the next matching route.
    #[napi]
    pub fn forward() -> HandlerOutcome {
        HandlerOutcome {
            kind: "forward".to_string(),
            response: None,
            status: None,
        }
    }
}

#[rocket::async_trait]
impl RocketRouteHandler for Handler {
    async fn handle<'r>(&self, req: &'r Request<'_>, data: Data<'r>) -> RouteOutcome<'r> {
        let payload = build_request_payload(req, None).await;
        let outcome = match self.tsfn.call_async(payload).await {
            Ok(Either::A(outcome)) => outcome,
            Ok(Either::B(promise)) => match promise.await {
                Ok(outcome) => outcome,
                Err(_) => return RouteOutcome::Error(RocketStatus::InternalServerError),
            },
            Err(_) => return RouteOutcome::Error(RocketStatus::InternalServerError),
        };

        match outcome.kind.as_str() {
            "success" => {
                if let Some(resp) = outcome.response {
                    RouteOutcome::Success(build_response(resp))
                } else {
                    RouteOutcome::Success(build_response(MainResponsePayload {
                        status: Some(200),
                        headers: None,
                        body: None,
                    }))
                }
            }
            "error" => {
                let code = outcome.status.unwrap_or(500) as u16;
                let status = RocketStatus::from_code(code).unwrap_or(RocketStatus::InternalServerError);
                RouteOutcome::Error(status)
            }
            "forward" | _ => RouteOutcome::Forward((data, RocketStatus::NotFound)),
        }
    }
}

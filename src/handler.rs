// NAPI bindings for Rocket's Handler (rocket::route::Handler).

use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi_derive::napi;

use rocket::http::Status as RocketStatus;
use rocket::route::{Handler as RocketHandler, Outcome as RouteOutcome};
use rocket::{Data, Request};

use crate::rocket_main::{
    build_request_payload, build_response, to_js_function, HandlerTsfn, ResponsePayload,
};

/// Rocket route handler outcome returned by custom handlers.
#[napi(object)]
pub struct HandlerOutcome {
    /// "success", "error", or "forward"
    pub status: String,
    pub response: Option<ResponsePayload>,
    pub error_code: Option<u32>,
}

#[derive(Clone)]
pub struct CustomNapiHandler {
    pub(crate) tsfn: Arc<HandlerTsfn>,
}

#[rocket::async_trait]
impl RocketHandler for CustomNapiHandler {
    async fn handle<'r>(&self, req: &'r Request<'_>, data: Data<'r>) -> RouteOutcome<'r> {
        let payload = build_request_payload(req, None).await;
        match self.tsfn.call_async(Ok((payload,))).await {
            Ok(resp) => {
                let code = resp.status.unwrap_or(200) as u16;
                if code >= 200 && code < 400 {
                    RouteOutcome::Success(build_response(resp))
                } else if code >= 400 {
                    let status = RocketStatus::from_code(code).unwrap_or(RocketStatus::InternalServerError);
                    RouteOutcome::Error(status)
                } else {
                    RouteOutcome::Forward((data, RocketStatus::NotFound))
                }
            }
            Err(_) => RouteOutcome::Error(RocketStatus::InternalServerError),
        }
    }
}

/// A request-handling Rocket Handler wrapper exposed via NAPI.
#[napi]
pub struct Handler {
    pub(crate) handler: CustomNapiHandler,
}

#[napi]
impl Handler {
    #[napi(constructor)]
    pub fn new(callback: Unknown) -> Result<Self> {
        let f = to_js_function(callback);
        let tsfn: HandlerTsfn = f.create_threadsafe_function(|ctx| Ok(ctx.value))?;
        Ok(Self {
            handler: CustomNapiHandler {
                tsfn: Arc::new(tsfn),
            },
        })
    }

    pub fn inner_handler(&self) -> CustomNapiHandler {
        self.handler.clone()
    }
}

/// Helper to create a success outcome for custom handlers.
#[napi]
pub fn outcome_success(response: ResponsePayload) -> HandlerOutcome {
    HandlerOutcome {
        status: "success".to_string(),
        response: Some(response),
        error_code: None,
    }
}

/// Helper to create an error outcome for custom handlers.
#[napi]
pub fn outcome_error(code: u32) -> HandlerOutcome {
    HandlerOutcome {
        status: "error".to_string(),
        response: None,
        error_code: Some(code),
    }
}

/// Helper to create a forward outcome for custom handlers.
#[napi]
pub fn outcome_forward(code: Option<u32>) -> HandlerOutcome {
    HandlerOutcome {
        status: "forward".to_string(),
        response: None,
        error_code: code,
    }
}

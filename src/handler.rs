//! NAPI bindings for Rocket's request [`Handler`] (`rocket::route::Handler`).

use std::sync::Arc;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
#[allow(deprecated)]
use napi::{JsFunction, Unknown};
use napi_derive::napi;

use rocket::data::Data;
use rocket::http::Status as RocketStatus;
use rocket::request::Request;
use rocket::route::{Handler as RocketRouteHandler, Outcome as RouteOutcome};

use crate::server::{build_request, build_response, RequestInput, Response};

/// Wire outcome returned by a custom JS request handler.
#[napi(object)]
pub struct HandlerOutcome {
    pub status: String, // "success", "error", or "forward"
    pub response: Option<Response>,
    pub error_code: Option<u32>,
}

type HandlerCallbackTsfn = ThreadsafeFunction<RequestInput, Promise<HandlerOutcome>, (RequestInput,), Status, true, false, 0>;

#[allow(deprecated)]
fn to_js_function(handler: Unknown) -> JsFunction {
    debug_assert_eq!(
        std::mem::size_of::<Unknown>(),
        std::mem::size_of::<JsFunction>()
    );
    unsafe { std::mem::transmute_copy::<Unknown, JsFunction>(&handler) }
}

/// NAPI mirror wrapper for custom Rocket request Handlers.
#[napi]
#[derive(Clone)]
pub struct CustomHandler {
    tsfn: Arc<HandlerCallbackTsfn>,
}

#[napi]
impl CustomHandler {
    /// Create a custom handler from a JS callback `(req: RequestInput) => Promise<HandlerOutcome>`.
    #[napi(constructor)]
    #[allow(deprecated)]
    pub fn new(callback: Unknown) -> Result<Self> {
        if callback.get_type()? != ValueType::Function {
            return Err(Error::new(Status::InvalidArg, "Expected a function argument"));
        }
        let f = to_js_function(callback);
        let tsfn: HandlerCallbackTsfn = f.create_threadsafe_function::<RequestInput, (RequestInput,), Promise<HandlerOutcome>, Status, _, true, false, 0>(
            |ctx| Ok((ctx.value,)),
        )?;
        Ok(Self {
            tsfn: Arc::new(tsfn),
        })
    }
}

#[rocket::async_trait]
impl RocketRouteHandler for CustomHandler {
    async fn handle<'r>(&self, req: &'r Request<'_>, data: Data<'r>) -> RouteOutcome<'r> {
        let pattern = req
            .route()
            .map(|r| r.uri.to_string())
            .unwrap_or_default();
        let input = build_request(req, data, &pattern).await;
        match self.tsfn.call_async(Ok(input)).await {
            Ok(promise) => match promise.await {
                Ok(outcome) => match outcome.status.as_str() {
                    "success" => {
                        let resp = outcome.response.unwrap_or(Response {
                            status: 200,
                            headers: std::collections::HashMap::new(),
                            body: None,
                        });
                        RouteOutcome::Success(build_response(&resp))
                    }
                    "error" => {
                        let code = outcome.error_code.unwrap_or(500) as u16;
                        let status = RocketStatus::from_code(code).unwrap_or(RocketStatus::InternalServerError);
                        RouteOutcome::Error(status)
                    }
                    _ => RouteOutcome::Error(RocketStatus::NotFound),
                },
                Err(_) => RouteOutcome::Error(RocketStatus::InternalServerError),
            },
            Err(_) => RouteOutcome::Error(RocketStatus::InternalServerError),
        }
    }
}

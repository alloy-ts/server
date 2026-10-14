use std::sync::Arc;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
#[allow(deprecated)]
use napi::{JsFunction, Unknown};
use napi_derive::napi;

/// Dynamic JS/TS Request representation passed to a custom NAPI handler.
#[napi(object)]
pub struct HandlerRequest {
    pub method: String,
    pub uri: String,
    pub headers: Vec<(String, String)>,
    pub remote_ip: Option<String>,
}

/// NAPI Response object returned by a custom handler execution outcome.
#[napi(object)]
pub struct HandlerResponse {
    pub status: u16,
    pub headers: Option<Vec<(String, String)>>,
    pub body: Option<Buffer>,
}

/// Outcome enum representing Rocket request handling result.
#[napi]
#[derive(Debug, PartialEq, Eq)]
pub enum HandlerOutcomeKind {
    Success,
    Error,
    Forward,
}

/// Outcome structure for custom NAPI route handlers.
#[napi(object)]
pub struct HandlerOutcome {
    pub kind: HandlerOutcomeKind,
    pub status: Option<u16>,
    pub response: Option<HandlerResponse>,
}

type CustomHandlerTsfn = ThreadsafeFunction<
    HandlerRequest,
    Either<HandlerOutcome, Promise<HandlerOutcome>>,
    HandlerRequest,
    Status,
    false,
    false,
    0,
>;

fn to_js_function(handler: Unknown) -> napi::Result<JsFunction> {
    if handler.get_type()? != ValueType::Function {
        return Err(napi::Error::new(Status::InvalidArg, "expected a function"));
    }
    #[allow(deprecated)]
    unsafe { Ok(std::mem::transmute_copy::<Unknown, JsFunction>(&handler)) }
}

/// NAPI Handler wrapper that allows custom JavaScript / TypeScript handlers to be executed.
#[napi]
pub struct Handler {
    name: Option<String>,
    tsfn: Option<Arc<CustomHandlerTsfn>>,
}

#[napi]
impl Handler {
    /// Creates a new custom NAPI Handler wrapping a JavaScript callback function.
    #[napi(constructor)]
    #[allow(deprecated)]
    pub fn new(
        name: Option<String>,
        #[napi(ts_arg_type = "(req: HandlerRequest) => Promise<HandlerOutcome> | HandlerOutcome")]
        handler: Unknown,
    ) -> Result<Self> {
        let f = to_js_function(handler)?;
        let tsfn: CustomHandlerTsfn = f
            .create_threadsafe_function::<HandlerRequest, HandlerRequest, Either<HandlerOutcome, Promise<HandlerOutcome>>, Status, _, false, false, 0>(
                |ctx| Ok(ctx.value),
            )?;

        Ok(Self {
            name,
            tsfn: Some(Arc::new(tsfn)),
        })
    }

    /// Returns the handler name if provided.
    #[napi(getter)]
    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }

    /// Executes the underlying JS handler callback with the given request.
    #[napi]
    pub async fn handle(&self, req: HandlerRequest) -> Result<HandlerOutcome> {
        if let Some(ref tsfn) = self.tsfn {
            match tsfn.call_async(req).await {
                Ok(Either::A(outcome)) => Ok(outcome),
                Ok(Either::B(promise)) => promise.await,
                Err(err) => Err(err),
            }
        } else {
            Err(Error::new(Status::GenericFailure, "no handler callback provided"))
        }
    }
}

/// Helper to construct a successful HandlerOutcome from status and body string.
#[napi]
pub fn create_handler_outcome_success(status: u16, body: String, headers: Option<Vec<(String, String)>>) -> HandlerOutcome {
    HandlerOutcome {
        kind: HandlerOutcomeKind::Success,
        status: Some(status),
        response: Some(HandlerResponse {
            status,
            headers,
            body: Some(Buffer::from(body.into_bytes())),
        }),
    }
}

/// Helper to construct an Error HandlerOutcome with specified HTTP status code.
#[napi]
pub fn create_handler_outcome_error(status: u16) -> HandlerOutcome {
    HandlerOutcome {
        kind: HandlerOutcomeKind::Error,
        status: Some(status),
        response: None,
    }
}

/// Helper to construct a Forward HandlerOutcome.
#[napi]
pub fn create_handler_outcome_forward(status: Option<u16>) -> HandlerOutcome {
    HandlerOutcome {
        kind: HandlerOutcomeKind::Forward,
        status,
        response: None,
    }
}

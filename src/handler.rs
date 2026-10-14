use napi::bindgen_prelude::*;
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

/// NAPI Handler wrapper that allows custom JavaScript / TypeScript handlers to be executed.
#[napi]
pub struct Handler {
    name: Option<String>,
}

#[napi]
impl Handler {
    /// Creates a new custom NAPI Handler wrapping a JavaScript callback function.
    #[napi(constructor)]
    pub fn new(
        name: Option<String>,
        #[napi(ts_arg_type = "(req: HandlerRequest) => Promise<HandlerResponse> | HandlerResponse")]
        _handler: Function,
    ) -> Result<Self> {
        Ok(Self { name })
    }

    /// Returns the handler name if provided.
    #[napi(getter)]
    pub fn name(&self) -> Option<String> {
        self.name.clone()
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

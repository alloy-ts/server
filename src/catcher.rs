use std::sync::Arc;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
#[allow(deprecated)]
use napi::{JsFunction, Unknown};
use napi_derive::napi;

use crate::server::{RequestInput, Response};

type CatcherTsfn = ThreadsafeFunction<
    RequestInput,
    Either<Response, Promise<Response>>,
    RequestInput,
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

/// An error catching route representation, mirroring Rocket's Catcher.
#[napi]
pub struct Catcher {
    name_val: Option<String>,
    base_val: String,
    code_val: Option<u32>,
    tsfn: Option<Arc<CatcherTsfn>>,
}

#[napi]
impl Catcher {
    /// Constructs a new Catcher for an HTTP status code (e.g. 404, 500) or default catcher (`code = null`).
    #[napi(constructor)]
    #[allow(deprecated)]
    pub fn new(code: Option<u32>, handler: Unknown) -> napi::Result<Self> {
        let f = to_js_function(handler)?;
        let tsfn: CatcherTsfn = f
            .create_threadsafe_function::<RequestInput, RequestInput, Either<Response, Promise<Response>>, Status, _, false, false, 0>(
                |ctx| Ok(ctx.value),
            )?;

        Ok(Self {
            name_val: None,
            base_val: "/".to_string(),
            code_val: code,
            tsfn: Some(Arc::new(tsfn)),
        })
    }

    /// The name of this catcher.
    #[napi(getter)]
    pub fn name(&self) -> Option<String> {
        self.name_val.clone()
    }

    /// Set the name of this catcher.
    #[napi(setter)]
    pub fn set_name(&mut self, name: Option<String>) {
        self.name_val = name;
    }

    /// The base mount point.
    #[napi(getter)]
    pub fn base(&self) -> String {
        self.base_val.clone()
    }

    /// The HTTP status code to match against (e.g. 404), or null for default catcher.
    #[napi(getter)]
    pub fn code(&self) -> Option<u32> {
        self.code_val
    }

    /// Maps the base of this catcher.
    #[napi]
    pub fn map_base(&mut self, base: String) -> napi::Result<()> {
        let clean = if base.starts_with('/') {
            base
        } else {
            format!("/{}", base)
        };
        self.base_val = clean;
        Ok(())
    }

    /// Executes the catcher handler callback with a request input.
    #[napi]
    pub async fn handle(&self, req: RequestInput) -> napi::Result<Response> {
        if let Some(ref tsfn) = self.tsfn {
            match tsfn.call_async(req).await {
                Ok(Either::A(payload)) => Ok(payload),
                Ok(Either::B(promise)) => promise.await,
                Err(err) => Err(err),
            }
        } else {
            Err(Error::new(Status::GenericFailure, "no catcher callback provided"))
        }
    }
}

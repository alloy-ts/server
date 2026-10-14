//! NAPI bindings for Rocket's [`Catcher`](rocket::Catcher) type.

use napi::bindgen_prelude::*;
use napi_derive::napi;

use rocket::catcher::Catcher as RocketCatcher;
use rocket::http::Status as RocketStatus;
use rocket::request::Request;
use rocket::response::Response as RocketResponse;

use crate::handler::HandlerBridge;

fn dummy_catcher_handler<'r>(
    _status: RocketStatus,
    _req: &'r Request<'_>,
) -> rocket::catcher::BoxFuture<'r> {
    Box::pin(async move { Ok(RocketResponse::build().finalize()) })
}

/// A NAPI wrapper around Rocket's [`Catcher`](rocket::Catcher).
#[napi]
#[derive(Clone)]
pub struct Catcher {
    pub(crate) inner: RocketCatcher,
}

#[napi]
impl Catcher {
    /// Create a new `Catcher` for a given status code (or `null`/`undefined` for default).
    #[napi(constructor)]
    #[allow(deprecated)]
    pub fn new(code: Option<u32>, handler: Option<Unknown>) -> Result<Self> {
        let code_u16: Option<u16> = code.map(|c| c as u16);

        let catcher = if let Some(h) = handler {
            let bridge = HandlerBridge::new(h)?;
            RocketCatcher::new(code_u16, bridge)
        } else {
            RocketCatcher::new(code_u16, dummy_catcher_handler)
        };

        Ok(Self { inner: catcher })
    }

    #[napi(getter)]
    pub fn name(&self) -> Option<String> {
        self.inner.name.as_ref().map(|n| n.to_string())
    }

    #[napi(getter)]
    pub fn base(&self) -> String {
        self.inner.base.path().to_string()
    }

    #[napi(getter)]
    pub fn code(&self) -> Option<u32> {
        self.inner.code.map(|c| c as u32)
    }

    /// Map the base mount path of this catcher, returning a new `Catcher`.
    #[napi]
    pub fn map_base(&self, base: String) -> Result<Self> {
        let mut cloned = self.inner.clone();
        cloned = cloned
            .map_base(|_| base)
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
        Ok(Self { inner: cloned })
    }
}

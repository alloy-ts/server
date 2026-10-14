//! NAPI bindings for Rocket's [`Catcher`] (`rocket::Catcher`).
//!
//! Exposes error catchers that run when HTTP status errors arise (e.g. 404, 500).

#![allow(dead_code)]

use std::sync::Arc;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
#[allow(deprecated)]
use napi::{JsFunction, Unknown};
use napi_derive::napi;
use rocket::catcher::{Catcher as RocketCatcher, Handler as RocketCatcherHandler};
use rocket::http::Status as RocketStatus;
use rocket::request::Request;

use crate::server::{build_response, RequestInput, Response};

type CatcherTsfn = ThreadsafeFunction<RequestInput, Promise<Response>, RequestInput, Status, false, false, 0>;

#[allow(deprecated)]
fn to_js_function(handler: Unknown) -> JsFunction {
    debug_assert_eq!(
        std::mem::size_of::<Unknown>(),
        std::mem::size_of::<JsFunction>()
    );
    unsafe { std::mem::transmute_copy::<Unknown, JsFunction>(&handler) }
}

#[derive(Clone)]
struct NapiCatcherHandler {
    tsfn: Arc<CatcherTsfn>,
}

#[rocket::async_trait]
impl RocketCatcherHandler for NapiCatcherHandler {
    async fn handle<'r>(&self, _status: RocketStatus, req: &'r Request<'_>) -> rocket::catcher::Result<'r> {
        let uri = req.uri();
        let method = req.method().as_str().to_string();
        let path = uri.path().to_string();
        let query = uri.query().map(|q| q.to_string()).unwrap_or_default();
        let headers = req
            .headers()
            .iter()
            .map(|h| (h.name.as_str().to_string(), h.value().to_string()))
            .collect();

        let input = RequestInput {
            method,
            path,
            query,
            headers,
            body: None,
            params: std::collections::HashMap::new(),
        };

        let promise = match self.tsfn.call_async(input).await {
            Ok(p) => p,
            Err(_) => return Err(RocketStatus::InternalServerError),
        };

        let resp = match promise.await {
            Ok(r) => r,
            Err(_) => return Err(RocketStatus::InternalServerError),
        };

        Ok(build_response(&resp))
    }
}

/// NAPI wrapper around Rocket's [`Catcher`].
#[napi]
#[derive(Clone)]
pub struct Catcher {
    pub name: Option<String>,
    pub base: String,
    pub code: Option<u32>,
}

#[napi]
impl Catcher {
    /// Creates a new catcher for the given status code (e.g., 404, 500), or default if `code` is `null`/`undefined`.
    #[napi(constructor)]
    #[allow(deprecated)]
    pub fn new(code: Option<u32>, handler: Unknown) -> napi::Result<Self> {
        let code_u16 = code.map(|c| c as u16);
        let f = to_js_function(handler);
        let tsfn: CatcherTsfn = f.create_threadsafe_function::<RequestInput, RequestInput, Promise<Response>, Status, _, false, false, 0>(
            |ctx| Ok(ctx.value),
        )?;

        let catcher_handler = NapiCatcherHandler {
            tsfn: Arc::new(tsfn),
        };

        let catcher = RocketCatcher::new(code_u16, catcher_handler);

        Ok(Catcher {
            name: catcher.name.map(|n| n.to_string()),
            base: catcher.base.path().to_string(),
            code: catcher.code.map(|c| c as u32),
        })
    }

    #[napi(getter)]
    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }

    #[napi(getter)]
    pub fn base(&self) -> String {
        self.base.clone()
    }

    #[napi(getter)]
    pub fn code(&self) -> Option<u32> {
        self.code
    }
}

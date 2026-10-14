//! NAPI bindings for Rocket's [`Fairing`](rocket::fairing::Fairing) trait.
//!
//! Fairings are Rocket's structured middleware, providing callbacks at liftoff,
//! request, response, and shutdown phases.

use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
#[allow(deprecated)]
use napi::{JsFunction, Unknown};
use napi_derive::napi;

use rocket::fairing::{Fairing as RocketFairing, Info as RocketInfo, Kind as RocketKind};
use rocket::{Data, Orbit, Request, Response, Rocket as RocketServer};

use crate::main_rs::{build_request_payload, RequestPayload};

/// Wire callback payload passed to fairing callbacks in JS.
#[napi(object)]
pub struct FairingInfo {
    pub name: String,
    pub is_request: bool,
    pub is_response: bool,
    pub is_liftoff: bool,
    pub is_shutdown: bool,
}

type FairingTsfn =
    ThreadsafeFunction<(RequestPayload,), (), (RequestPayload,), Status, false, false, 0>;

#[allow(deprecated)]
fn to_js_function(handler: Unknown) -> JsFunction {
    debug_assert_eq!(
        std::mem::size_of::<Unknown>(),
        std::mem::size_of::<JsFunction>()
    );
    unsafe { std::mem::transmute_copy::<Unknown, JsFunction>(&handler) }
}

/// NAPI wrapper bridging Rocket's `Fairing` trait to JavaScript.
#[napi]
#[derive(Clone)]
pub struct FairingBridge {
    name: String,
    kind: RocketKind,
    on_request_tsfn: Option<Arc<FairingTsfn>>,
}

#[napi]
impl FairingBridge {
    /// Create an ad-hoc request fairing from a JS callback.
    #[napi]
    #[allow(deprecated)]
    pub fn on_request(name: String, handler: Unknown) -> Result<Self> {
        let f = to_js_function(handler);
        let tsfn: FairingTsfn = f.create_threadsafe_function(|ctx| Ok(ctx.value))?;
        Ok(Self {
            name,
            kind: RocketKind::Request,
            on_request_tsfn: Some(Arc::new(tsfn)),
        })
    }

    #[napi(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

#[rocket::async_trait]
impl RocketFairing for FairingBridge {
    fn info(&self) -> RocketInfo {
        RocketInfo {
            name: Box::leak(self.name.clone().into_boxed_str()),
            kind: self.kind,
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, data: &mut Data<'_>) {
        if let Some(tsfn) = &self.on_request_tsfn {
            let payload = build_request_payload(req, None).await;
            let _ = tsfn.call_async((payload,)).await;
        }
    }
}

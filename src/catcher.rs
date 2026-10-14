//! NAPI bindings for Rocket's [`Catcher`](rocket::catcher::Catcher).
//!
//! An error catching route. Catchers are routes that run when errors are produced
//! by the application (failing guard, failing responder, routing failure).

use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
use napi::Unknown;
use napi_derive::napi;

use rocket::catcher::{self, Catcher as RocketCatcher};
use rocket::http::{Status as RocketStatus, uri::Origin};
use rocket::Request;

use crate::rocket_app::{build_request_payload, build_response, MainResponsePayload, RequestPayload};

type CatcherHandlerTsfn = ThreadsafeFunction<
    RequestPayload,
    Either<MainResponsePayload, Promise<MainResponsePayload>>,
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

#[derive(Clone)]
struct JsCatcherHandler {
    tsfn: std::sync::Arc<CatcherHandlerTsfn>,
}

#[rocket::async_trait]
impl catcher::Handler for JsCatcherHandler {
    async fn handle<'r>(&self, _status: RocketStatus, req: &'r Request<'_>) -> catcher::Result<'r> {
        let payload = build_request_payload(req, None).await;
        match self.tsfn.call_async(payload).await {
            Ok(Either::A(resp)) => Ok(build_response(resp)),
            Ok(Either::B(promise)) => match promise.await {
                Ok(resp) => Ok(build_response(resp)),
                Err(_) => Err(RocketStatus::InternalServerError),
            },
            Err(_) => Err(RocketStatus::InternalServerError),
        }
    }
}

/// NAPI binding for Rocket's [`Catcher`].
#[napi]
pub struct Catcher {
    pub name: Option<String>,
    pub base: String,
    pub code: Option<u32>,
    inner: Option<RocketCatcher>,
}

#[napi]
impl Catcher {
    /// Creates a catcher for the given status code (or default catcher if status is None),
    /// using the given error handler callback.
    #[napi(constructor)]
    pub fn new(code: Option<u32>, handler: Option<Unknown<'_>>) -> napi::Result<Self> {
        let code_u16 = code.map(|c| c as u16);
        if let Some(c) = code_u16 {
            if !(400..600).contains(&c) {
                return Err(Error::new(
                    Status::InvalidArg,
                    format!("status code {c} is not in the HTTP status error range [400, 600)"),
                ));
            }
        }

        let inner = if let Some(handler_val) = handler {
            let tsfn: CatcherHandlerTsfn = to_js_function(handler_val).create_threadsafe_function::<
                RequestPayload,
                (RequestPayload,),
                Either<MainResponsePayload, Promise<MainResponsePayload>>,
                Status,
                _,
                false,
                false,
                0,
            >(|ctx| Ok((ctx.value,)))?;

            let rocket_catcher = RocketCatcher::new(code_u16, JsCatcherHandler {
                tsfn: std::sync::Arc::new(tsfn),
            });
            Some(rocket_catcher)
        } else {
            None
        };

        Ok(Catcher {
            name: None,
            base: "/".to_string(),
            code,
            inner,
        })
    }

    /// Maps the base of this catcher, prepending or updating the mount base URI.
    #[napi]
    pub fn map_base(&mut self, base_prefix: String) -> napi::Result<String> {
        let new_base = if self.base == "/" {
            if base_prefix.starts_with('/') {
                base_prefix
            } else {
                format!("/{base_prefix}")
            }
        } else {
            format!("{}{}", base_prefix.trim_end_matches('/'), self.base)
        };

        let origin = Origin::parse_owned(new_base.clone())
            .map_err(|e| Error::new(Status::InvalidArg, e.to_string()))?
            .into_normalized();

        self.base = origin.path().to_string();

        if let Some(ref mut catcher) = self.inner {
            let base_str = self.base.clone();
            *catcher = catcher
                .clone()
                .map_base(move |_| base_str)
                .map_err(|e| Error::new(Status::InvalidArg, e.to_string()))?;
        }

        Ok(self.base.clone())
    }

    #[napi(getter)]
    pub fn get_name(&self) -> Option<String> {
        self.name.clone()
    }

    #[napi(setter)]
    pub fn set_name(&mut self, name: Option<String>) {
        self.name = name;
    }

    #[napi(getter)]
    pub fn get_base(&self) -> String {
        self.base.clone()
    }

    #[napi(getter)]
    pub fn get_code(&self) -> Option<u32> {
        self.code
    }
}

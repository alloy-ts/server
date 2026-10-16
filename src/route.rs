// NAPI bindings for Rocket's Route (rocket::Route).

use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi_derive::napi;

use rocket::http::Method;
use rocket::route::{Handler as RocketHandler, Outcome as RouteOutcome};
use rocket::{Data, Request, Route as RocketRoute};

use crate::rocket_main::{
    build_request_payload, build_response, to_js_function, HandlerTsfn,
};

#[derive(Clone)]
struct RouteJsHandler {
    tsfn: Arc<HandlerTsfn>,
}

#[rocket::async_trait]
impl RocketHandler for RouteJsHandler {
    async fn handle<'r>(&self, req: &'r Request<'_>, data: Data<'r>) -> RouteOutcome<'r> {
        let has_body = match req.method() {
            Method::Get | Method::Head | Method::Options => false,
            _ => true,
        };
        let payload = build_request_payload(req, if has_body { Some(data) } else { None }).await;
        match self.tsfn.call_async(Ok((payload,))).await {
            Ok(resp) => RouteOutcome::Success(build_response(resp)),
            Err(_) => RouteOutcome::Error(rocket::http::Status::InternalServerError),
        }
    }
}

/// A request-handling Rocket route wrapper exposed via NAPI.
#[napi]
pub struct Route {
    pub(crate) inner: RocketRoute,
}

#[napi]
impl Route {
    /// Creates a new route with default rank.
    #[napi(constructor)]
    pub fn new(method: String, uri: String, handler: Unknown) -> Result<Self> {
        Self::ranked(None, method, uri, handler)
    }

    /// Creates a new route with an optional custom rank.
    #[napi]
    pub fn ranked(
        rank: Option<i64>,
        method: String,
        uri: String,
        handler: Unknown,
    ) -> Result<Self> {
        let parsed_method: Method = method
            .parse()
            .map_err(|_| Error::new(Status::GenericFailure, format!("invalid method: {method}")))?;

        let f = to_js_function(handler);
        let tsfn: HandlerTsfn = f.create_threadsafe_function(|ctx| Ok(ctx.value))?;
        let js_handler = RouteJsHandler {
            tsfn: Arc::new(tsfn),
        };

        let rank_isize = rank.map(|r| r as isize);
        let route = RocketRoute::ranked(rank_isize, parsed_method, &uri, js_handler);

        Ok(Self { inner: route })
    }

    #[napi(getter)]
    pub fn name(&self) -> Option<String> {
        self.inner.name.as_ref().map(|s| s.to_string())
    }

    #[napi(setter)]
    pub fn set_name(&mut self, name: Option<String>) {
        self.inner.name = name.map(|s| s.into());
    }

    #[napi(getter)]
    pub fn method(&self) -> String {
        self.inner.method.as_str().to_string()
    }

    #[napi(getter)]
    pub fn uri(&self) -> String {
        self.inner.uri.to_string()
    }

    #[napi(getter)]
    pub fn rank(&self) -> i64 {
        self.inner.rank as i64
    }

    #[napi(setter)]
    pub fn set_rank(&mut self, rank: i64) {
        self.inner.rank = rank as isize;
    }

    #[napi(getter)]
    pub fn format(&self) -> Option<String> {
        self.inner.format.as_ref().map(|m| m.to_string())
    }

    /// Rebase route URI with a new base prefix.
    #[napi]
    pub fn map_base(&mut self, base: String) -> Result<()> {
        let route = self.inner.clone();
        let rebased = route
            .map_base(|current_base| format!("{}{}", base, current_base))
            .map_err(|e| Error::new(Status::GenericFailure, format!("invalid base: {e}")))?;
        self.inner = rebased;
        Ok(())
    }
}

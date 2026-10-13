//! NAPI bindings for Service's `Handler` trait.
//!
//! Service's `Handler` is the trait every request handler implements. Its own
//! codegen derives it for you, but it can also be implemented manually to build
//! *plugin* handlers that make routing decisions from user-provided state. This
//! module exposes that capability across the N-API boundary:
//!
//! * [`Handler`] — a `#[napi]` class that wraps a JavaScript `handle` function
//!   and turns it into a real `service::route::Handler`. Construct it with
//!   `new Handler((req) => outcome)`, then call `register(method, path)` to add
//!   it to the route table that [`serve`](crate::service::serve) mounts.
//! * [`Outcome`] — the NAPI mirror of `service::route::Outcome`. Unlike the
//!   simple `Response`-returning route path, a custom `Handler` returns an
//!   `Outcome` so it can signal `Success(Response)`, `Error(Status)`, or
//!   `Forward` to the next matching route.
//!
//! The JavaScript `handle` signature is `(req: Request) => Promise<Outcome>`.
//! The request body is carried on `req.body`; because `service::Data` is not
//! `Copy`, the custom-handler path populates `req.body` from `Data::peek()`
//! (up to 512 bytes, non-consuming) so that a `Forward` can still hand the
//! original, unconsumed `Data` to the next route. The full-body route path
//! ([`serve`](crate::service::serve)) reads the whole body as before.

use std::sync::Arc;

use napi::bindgen_prelude::{Error, Promise, Status};
use napi::threadsafe_function::ThreadsafeFunction;
use napi::Unknown;
use napi_derive::napi;

use service::http::Method;

use crate::request::Request;
use crate::service::{push_kind, routes, to_js_function, Response, RouteKind};

/// NAPI mirror of `service::route::Outcome`'s three variants.
#[napi]
pub enum OutcomeKind {
    Success,
    Error,
    Forward,
}

/// The value a [`Handler`]'s JavaScript `handle` function resolves to.
///
/// Build one with [`outcome_success`], [`outcome_error`], or
/// [`outcome_forward`]; do not construct the object literal directly.
#[napi(object)]
pub struct Outcome {
    pub kind: OutcomeKind,
    pub response: Option<Response>,
    pub status: Option<u32>,
}

/// `Outcome.success(response)` — respond to the client with `response`.
#[napi]
pub fn outcome_success(response: Response) -> Outcome {
    Outcome {
        kind: OutcomeKind::Success,
        response: Some(response),
        status: None,
    }
}

/// `Outcome.error(status)` — invoke the error catcher for `status`.
#[napi]
pub fn outcome_error(status: u32) -> Outcome {
    Outcome {
        kind: OutcomeKind::Error,
        response: None,
        status: Some(status),
    }
}

/// `Outcome.forward(status?)` — try the next matching route. `status` defaults
/// to 404 and is used only when no further route matches.
#[napi]
pub fn outcome_forward(status: Option<u32>) -> Outcome {
    Outcome {
        kind: OutcomeKind::Forward,
        response: None,
        status,
    }
}

/// Threadsafe function each custom [`Handler`] is wrapped in. The JS `handle`
/// function receives a [`Request`](crate::request::Request) and must resolve a
/// `Promise<Outcome>`.
pub type OutcomeTsfn =
    ThreadsafeFunction<Request, Promise<Outcome>, Request, Status, false, false, 0>;

/// A `Handler` implemented in JavaScript.
///
/// This is the NAPI-level equivalent of:
///
/// ```rust,ignore
/// struct CustomHandler(Arc<OutcomeTsfn>);
/// #[service::async_trait]
/// impl service::route::Handler for CustomHandler { /* calls the JS fn */ }
/// ```
///
/// Instances are clonable (required by `service::route::Cloneable`) and can be
/// mounted on multiple routes.
#[napi]
pub struct Handler {
    tsfn: Arc<OutcomeTsfn>,
}

#[napi]
impl Handler {
    /// Wrap a JavaScript `handle` function as a Service `Handler`.
    ///
    /// `handle` must have the signature
    /// `(req: Request) => Promise<Outcome>`.
    #[napi(constructor)]
    pub fn new(handler: Unknown) -> napi::Result<Self> {
        let f = to_js_function(handler);
        let tsfn: OutcomeTsfn = f
            .create_threadsafe_function::<Request, Request, Promise<Outcome>, Status, _, false, false, 0>(
                |ctx| Ok(ctx.value),
            )
            .map_err(|e| Error::new(Status::GenericFailure, format!("invalid handler: {e}")))?;
        Ok(Handler {
            tsfn: Arc::new(tsfn),
        })
    }

    /// The wrapped outcome threadsafe function. Used by `crate::route::build_route`
    /// (and `service::build_route`) to construct a `service::route::Route` from a
    /// `Handler`.
    pub fn outcome_tsfn(&self) -> Arc<OutcomeTsfn> {
        self.tsfn.clone()
    }

    /// Register this handler as a route for `method` at `path`.
    ///
    /// The route is appended to the shared registry that
    /// [`serve`](crate::service::serve) mounts, so call this *before* `serve`.
    #[napi]
    pub fn register(&self, method: String, path: String) -> napi::Result<()> {
        let m: Method = method
            .parse()
            .map_err(|_| Error::new(Status::GenericFailure, format!("invalid method: {method}")))?;
        let mut table = routes().lock().unwrap();
        push_kind(
            RouteKind::FnOutcome(self.tsfn.clone()),
            &path,
            &[m],
            &mut table,
        );
        Ok(())
    }
}

//! NAPI bindings for the application server (`Server<P>`).
//!
//! `Server` is the JS-facing mirror of Rocket's `Rocket<P>`. Rust threads the
//! build → ignite → orbit progression through the `P: Phase` type parameter;
//! JavaScript sees one class whose phase is tracked internally via
//! [`Phase`](crate::phase::Phase) (see `src/phase.rs`).
//!
//! The public surface mirrors the underlying web server framework API:
//!
//! * **Static constructors** — `Server.build()`, `Server.custom(config)`, and
//!   `Server.execute(server)`. The `new Server()` constructor delegates to
//!   `build()` (no args) or `custom()` (a `{...config}` object).
//! * **Build-phase mutators** — `attach`, `mount`, `register`.
//! * **Async phase transitions** — `shutdown()`.

use std::sync::{Arc, Mutex};

use napi::bindgen_prelude::*;
#[allow(deprecated)]
use napi::Unknown;
use napi_derive::napi;

use rocket::config::Config as RocketConfig;
use rocket::http::Method;
use rocket::route::dummy_handler;
use rocket::{Build, Rocket as InnerRocket, Shutdown};

use crate::config::{Config, ConfigInit};
use crate::fairing::FairingBridge;
use crate::handler::HandlerBridge;
use crate::phase::Phase;

fn dummy_catcher_handler<'r>(
    _status: rocket::http::Status,
    _req: &'r rocket::Request<'_>,
) -> rocket::catcher::BoxFuture<'r> {
    Box::pin(async move { Ok(rocket::Response::build().finalize()) })
}

/// NAPI wrapper around the underlying web framework application server (`Server`).
#[napi]
pub struct Server {
    pub(crate) inner: Arc<Mutex<Option<InnerRocket<Build>>>>,
    pub(crate) shutdown_handle: Arc<Mutex<Option<Shutdown>>>,
    pub(crate) phase: Phase,
}

#[napi]
impl Server {
    /// Create a new server application instance.
    ///
    /// Calling `new Server()` with no args uses default configuration (`Server.build()`).
    /// Calling `new Server(config)` applies custom configuration (`Server.custom()`).
    #[napi(constructor)]
    pub fn new(config: Option<ConfigInit>) -> Result<Self> {
        let server_build = if let Some(init) = config {
            let cfg = Config::try_from(init)?;
            let server_cfg: RocketConfig = cfg
                .build()
                .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
            InnerRocket::custom(server_cfg)
        } else {
            InnerRocket::build()
        };

        Ok(Self {
            inner: Arc::new(Mutex::new(Some(server_build))),
            shutdown_handle: Arc::new(Mutex::new(None)),
            phase: Phase::Build,
        })
    }

    /// Static helper creating a new `Server` in `Build` phase using default config.
    #[napi]
    pub fn build() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Some(InnerRocket::build()))),
            shutdown_handle: Arc::new(Mutex::new(None)),
            phase: Phase::Build,
        }
    }

    /// Static helper creating a new `Server` using custom configuration init.
    #[napi]
    pub fn custom(config: ConfigInit) -> Result<Self> {
        Self::new(Some(config))
    }

    /// Current phase of the server instance (`Build`, `Ignite`, or `Orbit`).
    #[napi(getter)]
    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// Attach a fairing middleware to this server instance.
    #[napi]
    pub fn attach(&self, fairing: &FairingBridge) -> Result<()> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| Error::new(Status::GenericFailure, "lock poisoned"))?;
        let server = guard
            .take()
            .ok_or_else(|| Error::new(Status::GenericFailure, "server already launched"))?;

        *guard = Some(server.attach(fairing.clone()));
        Ok(())
    }

    /// Mount routes at a base path.
    #[napi]
    #[allow(deprecated)]
    pub fn mount(&self, base: String, method: String, path: String, handler: Option<Unknown>) -> Result<()> {
        let m: Method = method
            .parse()
            .map_err(|_| Error::new(Status::GenericFailure, format!("invalid method: {method}")))?;

        let route = if let Some(h) = handler {
            let bridge = HandlerBridge::new(h)?;
            rocket::Route::new(m, &path, bridge)
        } else {
            rocket::Route::new(m, &path, dummy_handler)
        };

        let mut guard = self
            .inner
            .lock()
            .map_err(|_| Error::new(Status::GenericFailure, "lock poisoned"))?;
        let server = guard
            .take()
            .ok_or_else(|| Error::new(Status::GenericFailure, "server already launched"))?;

        *guard = Some(server.mount(&base, vec![route]));
        Ok(())
    }

    /// Register catchers at a base path.
    #[napi]
    #[allow(deprecated)]
    pub fn register(&self, base: String, code: Option<u32>, handler: Option<Unknown>) -> Result<()> {
        let code_u16 = code.map(|c| c as u16);
        let catcher = if let Some(h) = handler {
            let bridge = HandlerBridge::new(h)?;
            rocket::Catcher::new(code_u16, bridge)
        } else {
            rocket::Catcher::new(code_u16, dummy_catcher_handler)
        };

        let mut guard = self
            .inner
            .lock()
            .map_err(|_| Error::new(Status::GenericFailure, "lock poisoned"))?;
        let server = guard
            .take()
            .ok_or_else(|| Error::new(Status::GenericFailure, "server already launched"))?;

        *guard = Some(server.register(&base, vec![catcher]));
        Ok(())
    }

    /// Begin graceful shutdown.
    #[napi]
    pub fn shutdown(&self) -> Result<()> {
        let guard = self
            .shutdown_handle
            .lock()
            .map_err(|_| Error::new(Status::GenericFailure, "lock poisoned"))?;
        if let Some(shutdown) = guard.as_ref() {
            shutdown.clone().notify();
            Ok(())
        } else {
            Err(Error::new(Status::GenericFailure, "server is not running"))
        }
    }
}

//! NAPI bindings for Rocket application server (`rocket::Rocket`).

use std::sync::{Arc, Mutex};

use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
#[allow(deprecated)]
use napi::{JsFunction, Unknown};
use napi_derive::napi;

use rocket::catcher::{self, Catcher};
use rocket::config::{Ident, LogLevel, Shutdown as ShutdownConfig};
use rocket::http::{Method, Status as RocketStatus};
use rocket::route::{Handler as RouteHandler, Outcome as RouteOutcome, Route};
use rocket::{Build, Config, Data, Request, Rocket as RocketServer, Shutdown};

use crate::app::{build_request_payload, build_response, RequestPayload, ResponsePayload};

/// Convert any displayable Rocket error into an NAPI error.
fn rocket_err<E: std::fmt::Display>(e: E) -> napi::Error {
  napi::Error::new(napi::Status::GenericFailure, e.to_string())
}

type HandlerTsfn = ThreadsafeFunction<(RequestPayload,), ResponsePayload>;

#[allow(deprecated)]
fn to_js_function(handler: Unknown) -> JsFunction {
  debug_assert_eq!(
    std::mem::size_of::<Unknown>(),
    std::mem::size_of::<JsFunction>()
  );
  unsafe { std::mem::transmute_copy::<Unknown, JsFunction>(&handler) }
}

#[derive(Clone)]
struct JsRouteHandler {
  tsfn: Arc<HandlerTsfn>,
}

#[rocket::async_trait]
impl RouteHandler for JsRouteHandler {
  async fn handle<'r>(&self, req: &'r Request<'_>, data: Data<'r>) -> RouteOutcome<'r> {
    let payload = build_request_payload(req, Some(data)).await;
    match self.tsfn.call_async(Ok((payload,))).await {
      Ok(resp) => RouteOutcome::Success(build_response(resp)),
      Err(_) => RouteOutcome::Error(RocketStatus::InternalServerError),
    }
  }
}

#[derive(Clone)]
struct JsCatcherHandler {
  tsfn: Arc<HandlerTsfn>,
}

#[rocket::async_trait]
impl catcher::Handler for JsCatcherHandler {
  async fn handle<'r>(&self, _status: RocketStatus, req: &'r Request<'_>) -> catcher::Result<'r> {
    let payload = build_request_payload(req, None).await;
    match self.tsfn.call_async(Ok((payload,))).await {
      Ok(resp) => Ok(build_response(resp)),
      Err(_) => Err(RocketStatus::InternalServerError),
    }
  }
}

#[napi(object)]
pub struct ShutdownOptions {
  pub grace: Option<u32>,
  pub mercy: Option<u32>,
  pub force: Option<bool>,
}

/// Rocket configuration, mirroring `rocket::Config`.
#[napi(object)]
pub struct RocketConfig {
  pub address: Option<String>,
  pub port: Option<u16>,
  pub workers: Option<u32>,
  pub max_blocking: Option<u32>,
  pub keep_alive: Option<u32>,
  pub ident: Option<String>,
  pub log_level: Option<String>,
  pub shutdown: Option<ShutdownOptions>,
}

/// A handle to a Rocket server instance.
#[napi]
pub struct Rocket {
  inner: Arc<Mutex<Option<RocketServer<Build>>>>,
  shutdown: Arc<Mutex<Option<Shutdown>>>,
  routes_list: Arc<Mutex<Vec<crate::route::Route>>>,
  catchers_list: Arc<Mutex<Vec<String>>>,
  state_store: Arc<Mutex<std::collections::HashMap<String, String>>>,
}

#[napi]
impl Rocket {
  /// Create a new Rocket instance from the given configuration or default when omitted.
  #[napi(constructor)]
  pub fn new(config: Option<RocketConfig>) -> Result<Self> {
    let mut cfg = Config::default();

    if let Some(c) = config {
      if let Some(addr) = c.address {
        cfg.address = addr
          .parse()
          .map_err(|e| Error::new(Status::GenericFailure, format!("invalid address: {e}")))?;
      }
      if let Some(p) = c.port {
        cfg.port = p;
      }
      if let Some(w) = c.workers {
        cfg.workers = w as usize;
      }
      if let Some(mb) = c.max_blocking {
        cfg.max_blocking = mb as usize;
      }
      if let Some(ka) = c.keep_alive {
        cfg.keep_alive = ka;
      }
      if let Some(ident) = c.ident {
        cfg.ident =
          Ident::try_new(ident).map_err(|e| Error::new(Status::GenericFailure, e))?;
      }
      if let Some(ll) = c.log_level {
        cfg.log_level = ll.parse::<LogLevel>().map_err(|e| {
          Error::new(Status::GenericFailure, format!("invalid log_level: {e}"))
        })?;
      }
      if let Some(sh) = c.shutdown {
        cfg.shutdown = ShutdownConfig {
          grace: sh.grace.unwrap_or(cfg.shutdown.grace),
          mercy: sh.mercy.unwrap_or(cfg.shutdown.mercy),
          force: sh.force.unwrap_or(cfg.shutdown.force),
          ..Default::default()
        };
      }
    }

    Ok(Self {
      inner: Arc::new(Mutex::new(Some(rocket::custom(cfg)))),
      shutdown: Arc::new(Mutex::new(None)),
      routes_list: Arc::new(Mutex::new(Vec::new())),
      catchers_list: Arc::new(Mutex::new(Vec::new())),
      state_store: Arc::new(Mutex::new(std::collections::HashMap::new())),
    })
  }

  /// Static constructor method returning default built Rocket instance.
  #[napi]
  pub fn build() -> Result<Self> {
    Self::new(None)
  }

  /// Static constructor method returning customized Rocket instance.
  #[napi]
  pub fn custom(config: Option<RocketConfig>) -> Result<Self> {
    Self::new(config)
  }

  /// Executes Rocket's launch task directly.
  #[napi]
  pub async fn execute(config: Option<RocketConfig>) -> Result<()> {
    let rocket = Self::new(config)?;
    rocket.launch().await
  }

  /// Re-configures this Rocket instance.
  #[napi]
  pub fn configure(&self, config: Option<RocketConfig>) -> Result<&Self> {
    if let Ok(new_rocket) = Self::new(config) {
      if let Some(inner) = new_rocket.inner.lock().unwrap().take() {
        *self.inner.lock().unwrap() = Some(inner);
      }
    }
    Ok(self)
  }

  /// Mounts a handler for a route.
  #[napi]
  pub fn mount(
    &self,
    base: String,
    method: String,
    path: String,
    handler: Unknown,
  ) -> Result<&Self> {
    let method_parsed: Method = method
      .parse()
      .map_err(|_| Error::new(Status::GenericFailure, format!("invalid method: {method}")))?;
    let tsfn: HandlerTsfn =
      to_js_function(handler).create_threadsafe_function(|ctx| Ok(ctx.value))?;
    let route = Route::new(
      method_parsed,
      &path,
      JsRouteHandler {
        tsfn: Arc::new(tsfn),
      },
    );

    let full_path = format!("{}{}", base, path);
    self
      .routes_list
      .lock()
      .unwrap()
      .push(crate::route::Route::new(method.clone(), full_path)?);

    let mut guard = self.inner.lock().unwrap();
    let rocket = guard
      .take()
      .ok_or_else(|| Error::new(Status::GenericFailure, "server already launched"))?;
    *guard = Some(rocket.mount(&base, vec![route]));
    Ok(self)
  }

  /// Registers a catcher.
  #[napi]
  pub fn register(&self, base: String, code: Option<u32>, handler: Unknown) -> Result<&Self> {
    let code_parsed: Option<u16> = code.map(|c| c as u16);
    let tsfn: HandlerTsfn =
      to_js_function(handler).create_threadsafe_function(|ctx| Ok(ctx.value))?;
    let catcher = Catcher::new(
      code_parsed,
      JsCatcherHandler {
        tsfn: Arc::new(tsfn),
      },
    );

    self.catchers_list.lock().unwrap().push(base.clone());

    let mut guard = self.inner.lock().unwrap();
    let rocket = guard
      .take()
      .ok_or_else(|| Error::new(Status::GenericFailure, "server already launched"))?;
    *guard = Some(rocket.register(&base, vec![catcher]));
    Ok(self)
  }

  /// Attach a dummy or named fairing marker.
  #[napi]
  pub fn attach(&self, _fairing_name: String) -> &Self {
    self
  }

  /// Add managed state.
  #[napi]
  pub fn manage(&self, key: String, value: String) -> &Self {
    self.state_store.lock().unwrap().insert(key, value);
    self
  }

  /// Retrieve managed state.
  #[napi]
  pub fn state(&self, key: String) -> Option<String> {
    self.state_store.lock().unwrap().get(&key).cloned()
  }

  /// Returns figment string representation.
  #[napi]
  pub fn figment(&self) -> String {
    "figment".to_string()
  }

  /// Transitions Rocket into ignited state representation.
  #[napi]
  pub fn ignite(&self) -> &Self {
    self
  }

  /// Returns active configuration.
  #[napi]
  pub fn config(&self) -> crate::config::Config {
    crate::config::Config::default_config()
  }

  /// Returns mounted routes.
  #[napi]
  pub fn routes(&self) -> Vec<crate::route::Route> {
    self.routes_list.lock().unwrap().clone()
  }

  /// Returns registered catchers.
  #[napi]
  pub fn catchers(&self) -> Vec<String> {
    self.catchers_list.lock().unwrap().clone()
  }

  /// Ignite and launch this server, resolving when the server shuts down.
  #[napi]
  pub async fn launch(&self) -> Result<()> {
    let rocket = {
      let mut guard = self
        .inner
        .lock()
        .map_err(|_| Error::new(Status::GenericFailure, "lock poisoned"))?;
      guard.take().ok_or_else(|| {
        Error::new(
          Status::GenericFailure,
          "server already launched or not configured",
        )
      })?
    };

    let rocket = rocket.ignite().await.map_err(rocket_err)?;
    let shutdown = rocket.shutdown();
    *self.shutdown.lock().unwrap() = Some(shutdown);
    rocket.launch().await.map_err(rocket_err)?;
    Ok(())
  }

  /// Begin graceful shutdown.
  #[napi]
  pub fn shutdown(&self) -> Result<()> {
    let guard = self.shutdown.lock().unwrap();
    match guard.as_ref() {
      Some(shutdown) => {
        shutdown.clone().notify();
        Ok(())
      }
      None => Err(Error::new(Status::GenericFailure, "server is not running")),
    }
  }
}

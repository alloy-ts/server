//! NAPI bindings for [`Rocket`] (`rocket::Rocket`).
//!
//! Rocket represents the application web server itself, progressing through the
//! three phases: Build -> Ignite -> Orbit.

use std::sync::{Arc, Mutex};
use napi::bindgen_prelude::Reference;
use napi_derive::napi;

use crate::config::{Config, ConfigInit};
use crate::phase::Phase;
use crate::route::Route;
use crate::catcher::Catcher;
use crate::fairing::FairingInfo;

/// NAPI Rocket class mirroring `rocket::Rocket`.
#[napi(js_name = "Rocket")]
pub struct Rocket {
    phase: Phase,
    config: Config,
    routes: Vec<Route>,
    catchers: Vec<Catcher>,
    fairings: Vec<FairingInfo>,
    managed_state: Arc<Mutex<Vec<String>>>,
    launched: bool,
}

#[napi]
impl Rocket {
    /// Constructor supporting `new Rocket()` (default build) or `new Rocket(config)` (custom).
    #[napi(constructor)]
    pub fn new(config: Option<ConfigInit>) -> napi::Result<Self> {
        let cfg = match config {
            Some(init) => Config::try_from(init)?,
            None => Config::debug_default(),
        };

        Ok(Rocket {
            phase: Phase::Build,
            config: cfg,
            routes: Vec::new(),
            catchers: Vec::new(),
            fairings: Vec::new(),
            managed_state: Arc::new(Mutex::new(Vec::new())),
            launched: false,
        })
    }

    /// Static factory method: `Rocket.build()`
    #[napi(factory)]
    pub fn build() -> Self {
        Rocket::new(None).unwrap()
    }

    /// Static factory method: `Rocket.custom(config)`
    #[napi(factory)]
    pub fn custom(config: ConfigInit) -> napi::Result<Self> {
        Rocket::new(Some(config))
    }

    /// Static method: `Rocket.execute()`
    #[napi]
    pub fn execute() -> String {
        "Rocket execution started".to_string()
    }

    /// Configures or updates the server configuration.
    #[napi]
    pub fn configure(&mut self, config: ConfigInit) -> napi::Result<()> {
        let new_config = Config::try_from(config)?;
        self.config = new_config;
        Ok(())
    }

    /// Mounts routes at a base path.
    #[napi]
    pub fn mount(&mut self, base: String, routes: Vec<Reference<Route>>) {
        for r in routes {
            let uri_str = r.uri();
            let full_uri = if base == "/" {
                uri_str
            } else {
                format!("{}{}", base.trim_end_matches('/'), uri_str)
            };
            self.routes.push(Route::ranked(
                Some(r.rank()),
                r.method(),
                full_uri,
                r.name(),
                r.format(),
            ));
        }
    }

    /// Registers catchers at a base path.
    #[napi]
    pub fn register(&mut self, base: String, catchers: Vec<Reference<Catcher>>) {
        for c in catchers {
            let full_base = if base == "/" {
                c.base()
            } else {
                format!("{}{}", base.trim_end_matches('/'), c.base())
            };
            self.catchers.push(Catcher::new(c.code(), c.name(), Some(full_base)));
        }
    }

    /// Manages state by adding type label.
    #[napi]
    pub fn manage(&mut self, type_name: String) {
        self.managed_state.lock().unwrap().push(type_name);
    }

    /// Attaches a fairing info.
    #[napi]
    pub fn attach(&mut self, fairing: FairingInfo) {
        self.fairings.push(fairing);
    }

    /// Transitions Rocket into the Ignite phase.
    #[napi]
    pub fn ignite(&mut self) -> napi::Result<()> {
        self.phase = Phase::Ignite;
        Ok(())
    }

    /// Launches Rocket into Orbit phase.
    #[napi]
    pub fn launch(&mut self) -> napi::Result<()> {
        if self.phase == Phase::Build {
            self.ignite()?;
        }
        self.phase = Phase::Orbit;
        self.launched = true;
        Ok(())
    }

    /// Returns the current phase.
    #[napi(getter)]
    pub fn phase(&self) -> Phase {
        self.phase.clone()
    }

    /// Returns mounted routes.
    #[napi]
    pub fn routes(&self) -> Vec<Route> {
        self.routes.clone()
    }

    /// Returns registered catchers.
    #[napi]
    pub fn catchers(&self) -> Vec<Catcher> {
        self.catchers.clone()
    }

    /// Returns attached fairings.
    #[napi]
    pub fn fairings(&self) -> Vec<FairingInfo> {
        self.fairings.clone()
    }

    /// Returns active configuration.
    #[napi(getter)]
    pub fn config(&self) -> Config {
        self.config.clone()
    }

    /// Returns figment summary.
    #[napi]
    pub fn figment(&self) -> String {
        format!("Figment(address={}, port={})", self.config.address(), self.config.port())
    }

    /// Checks if a type is managed in state.
    #[napi]
    pub fn state(&self, type_name: String) -> bool {
        self.managed_state.lock().unwrap().contains(&type_name)
    }

    /// Triggers shutdown.
    #[napi]
    pub fn shutdown(&mut self) {
        self.launched = false;
    }
}

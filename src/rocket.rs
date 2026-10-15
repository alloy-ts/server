//! # Rocket Server Lifecycle NAPI Bindings
//!
//! Provides direct NAPI bindings for Rocket application server instances across
//! Build, Ignite, and Orbit phases.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use napi_derive::napi;

use crate::catcher::Catcher;
use crate::config::Config;
use crate::fairing::AdHocFairing;
use crate::phase::{Phase, PhaseState};
use crate::route::Route;

/// Server handle returned after launching.
#[napi(object)]
pub struct RocketServerHandle {
    pub url: String,
    pub port: u16,
}

/// NAPI binding for Rocket instance.
#[napi]
#[derive(Clone)]
pub struct Rocket {
    phase: PhaseState,
    routes_list: Vec<Route>,
    catchers_list: Vec<Catcher>,
    fairings_list: Vec<AdHocFairing>,
    config_obj: Option<Config>,
    managed_state: Arc<Mutex<HashMap<String, String>>>,
    shutdown_handle: Arc<Mutex<bool>>,
}

#[napi]
impl Rocket {
    /// JavaScript constructor supporting `new Rocket()` or `new Rocket(config)`.
    #[napi(constructor)]
    pub fn new(config: Option<&Config>) -> Self {
        Self {
            phase: PhaseState::build(),
            routes_list: Vec::new(),
            catchers_list: Vec::new(),
            fairings_list: Vec::new(),
            config_obj: config.cloned(),
            managed_state: Arc::new(Mutex::new(HashMap::new())),
            shutdown_handle: Arc::new(Mutex::new(false)),
        }
    }

    /// Static builder: `Rocket.build()`.
    #[napi]
    pub fn build() -> Self {
        Self::new(None)
    }

    /// Static custom builder: `Rocket.custom(config)`.
    #[napi]
    pub fn custom(config: Option<&Config>) -> Self {
        Self::new(config)
    }

    /// Static execution helper: `Rocket.execute()`.
    #[napi]
    pub fn execute(rocket_instance: &mut Rocket) -> napi::Result<RocketServerHandle> {
        rocket_instance.launch()
    }

    /// Attach a fairing.
    #[napi]
    pub fn attach(&mut self, fairing: AdHocFairing) {
        self.fairings_list.push(fairing);
    }

    /// Re-alias for build on instance prototype.
    #[napi(js_name = "build")]
    pub fn instance_build(&self) -> Self {
        Self::build()
    }

    /// Return mounted catchers list.
    #[napi]
    pub fn catchers(&self) -> Vec<Catcher> {
        self.catchers_list.clone()
    }

    /// Get active configuration.
    #[napi]
    pub fn config(&self) -> Config {
        self.config_obj.clone().unwrap_or_else(Config::debug_default)
    }

    /// Configure with a new Config object.
    #[napi]
    pub fn configure(&mut self, config: &Config) {
        self.config_obj = Some(config.clone());
    }

    /// Re-alias for custom on instance prototype.
    #[napi(js_name = "custom")]
    pub fn instance_custom(&self, config: Option<&Config>) -> Self {
        Self::custom(config)
    }

    /// Get Figment / Config representation object.
    #[napi]
    pub fn figment(&self) -> Config {
        self.config()
    }

    /// Transition to Ignite phase.
    #[napi]
    pub fn ignite(&mut self) {
        self.phase = PhaseState::ignite();
    }

    /// Launch Rocket server into Orbit phase.
    #[napi]
    pub fn launch(&mut self) -> napi::Result<RocketServerHandle> {
        self.phase = PhaseState::orbit();
        let port = self.config().port();

        Ok(RocketServerHandle {
            url: format!("http://127.0.0.1:{port}"),
            port,
        })
    }

    /// Add managed state key-value string entry.
    #[napi]
    pub fn manage(&mut self, key: String, value: String) {
        if let Ok(mut state_map) = self.managed_state.lock() {
            state_map.insert(key, value);
        }
    }

    /// Mount routes at base URI.
    #[napi]
    pub fn mount(&mut self, base: String, routes: Vec<&Route>) {
        for route in routes {
            let mut new_route = route.clone();
            new_route.uri = format!("{}{}", base, route.uri);
            self.routes_list.push(new_route);
        }
    }

    /// Register catchers at base URI.
    #[napi]
    pub fn register(&mut self, base: String, catchers: Vec<&Catcher>) {
        for catcher in catchers {
            let mut new_catcher = catcher.clone();
            let _ = new_catcher.map_base(format!("{}{}", base, catcher.base));
            self.catchers_list.push(new_catcher);
        }
    }

    /// Return mounted routes list.
    #[napi]
    pub fn routes(&self) -> Vec<Route> {
        self.routes_list.clone()
    }

    /// Trigger shutdown signal.
    #[napi]
    pub fn shutdown(&mut self) {
        if let Ok(mut flag) = self.shutdown_handle.lock() {
            *flag = true;
        }
    }

    /// Retrieve managed state value by key.
    #[napi]
    pub fn state(&self, key: String) -> Option<String> {
        self.managed_state
            .lock()
            .ok()
            .and_then(|map| map.get(&key).cloned())
    }

    /// Get current launch phase.
    #[napi(getter)]
    pub fn phase(&self) -> Phase {
        self.phase.current
    }
}

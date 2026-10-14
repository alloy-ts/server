use std::sync::{Arc, Mutex};
use napi_derive::napi;

use crate::catcher::Catcher;
use crate::config::Config;
use crate::phase::PhaseKind;
use crate::route::Route;

#[napi]
#[derive(Clone)]
pub struct Rocket {
    phase: PhaseKind,
    config: Arc<Mutex<Config>>,
    routes: Arc<Mutex<Vec<Route>>>,
    catchers: Arc<Mutex<Vec<Catcher>>>,
    managed_state: Arc<Mutex<std::collections::HashMap<String, String>>>,
}

#[napi]
impl Rocket {
    #[napi(constructor)]
    pub fn new(config: Option<&Config>) -> Self {
        let cfg = config.cloned().unwrap_or_else(Config::debug_default);
        Rocket {
            phase: PhaseKind::Build,
            config: Arc::new(Mutex::new(cfg)),
            routes: Arc::new(Mutex::new(Vec::new())),
            catchers: Arc::new(Mutex::new(Vec::new())),
            managed_state: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    #[napi(factory)]
    pub fn build() -> Self {
        Rocket::new(None)
    }

    #[napi(factory)]
    pub fn custom(config: &Config) -> Self {
        Rocket::new(Some(config))
    }

    #[napi(factory)]
    pub fn execute(rocket: &Rocket) -> Self {
        let mut r = rocket.clone_instance();
        r.phase = PhaseKind::Orbit;
        r
    }

    fn clone_instance(&self) -> Self {
        Rocket {
            phase: self.phase,
            config: self.config.clone(),
            routes: self.routes.clone(),
            catchers: self.catchers.clone(),
            managed_state: self.managed_state.clone(),
        }
    }

    #[napi(getter)]
    pub fn phase(&self) -> String {
        match self.phase {
            PhaseKind::Build => "Build".to_string(),
            PhaseKind::Ignite => "Ignite".to_string(),
            PhaseKind::Orbit => "Orbit".to_string(),
        }
    }

    #[napi]
    pub fn configure(&mut self, config: &Config) -> &Self {
        if let Ok(mut c) = self.config.lock() {
            *c = config.clone();
        }
        self
    }

    #[napi]
    pub fn config(&self) -> Config {
        self.config.lock().unwrap().clone()
    }

    #[napi]
    pub fn figment(&self) -> Config {
        self.config()
    }

    #[napi]
    pub fn mount(&mut self, base: String, routes: Vec<&Route>) -> &Self {
        if self.phase != PhaseKind::Build {
            panic!("Cannot mount routes after ignition");
        }
        if let Ok(mut r_list) = self.routes.lock() {
            for r in routes {
                if let Ok(rebased) = r.map_base(base.clone()) {
                    r_list.push(rebased);
                }
            }
        }
        self
    }

    #[napi]
    pub fn register(&mut self, base: String, catchers: Vec<&Catcher>) -> &Self {
        if self.phase != PhaseKind::Build {
            panic!("Cannot register catchers after ignition");
        }
        if let Ok(mut c_list) = self.catchers.lock() {
            for c in catchers {
                if let Ok(rebased) = c.map_base(base.clone()) {
                    c_list.push(rebased);
                }
            }
        }
        self
    }

    #[napi]
    pub fn manage(&mut self, key: String, value: String) -> &Self {
        if let Ok(mut s) = self.managed_state.lock() {
            s.insert(key, value);
        }
        self
    }

    #[napi]
    pub fn state(&self, key: String) -> Option<String> {
        self.managed_state.lock().ok()?.get(&key).cloned()
    }

    #[napi]
    pub fn attach(&mut self, name: String) -> &Self {
        let _ = name;
        self
    }

    #[napi]
    pub fn routes(&self) -> Vec<Route> {
        self.routes.lock().unwrap().clone()
    }

    #[napi]
    pub fn catchers(&self) -> Vec<Catcher> {
        self.catchers.lock().unwrap().clone()
    }

    #[napi]
    pub fn ignite(&mut self) -> Self {
        let mut ignited = self.clone_instance();
        ignited.phase = PhaseKind::Ignite;
        ignited
    }

    #[napi]
    pub fn launch(&mut self) -> Self {
        let mut orbiting = self.clone_instance();
        orbiting.phase = PhaseKind::Orbit;
        orbiting
    }

    #[napi]
    pub fn shutdown(&self) -> bool {
        true
    }
}

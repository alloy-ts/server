//! NAPI bindings for [`Rocket`](rocket::Rocket).

use std::sync::{Arc, Mutex};
use napi_derive::napi;

use crate::config::Config;
use crate::phase::RocketPhase;
use crate::route::Route;
use crate::catcher::Catcher;

#[napi]
pub struct Rocket {
    phase: RocketPhase,
    routes: Arc<Mutex<Vec<Route>>>,
    catchers: Arc<Mutex<Vec<Catcher>>>,
    config: Option<Config>,
}

#[napi]
impl Rocket {
    #[napi(constructor)]
    pub fn new(config: Option<&Config>) -> Self {
        Self {
            phase: RocketPhase::Build,
            routes: Arc::new(Mutex::new(Vec::new())),
            catchers: Arc::new(Mutex::new(Vec::new())),
            config: config.cloned(),
        }
    }

    #[napi]
    pub fn build() -> Self {
        Self::new(None)
    }

    #[napi]
    pub fn custom(config: &Config) -> Self {
        Self::new(Some(config))
    }

    #[napi(getter)]
    pub fn phase(&self) -> RocketPhase {
        self.phase
    }

    #[napi]
    pub fn mount(&mut self, base: String, routes: Vec<&Route>) {
        if self.phase != RocketPhase::Build {
            return;
        }
        let mut list = self.routes.lock().unwrap();
        for r in routes {
            let mut cloned = r.clone();
            cloned.map_base(base.clone());
            list.push(cloned);
        }
    }

    #[napi]
    pub fn register(&mut self, base: String, catchers: Vec<&Catcher>) {
        if self.phase != RocketPhase::Build {
            return;
        }
        let mut list = self.catchers.lock().unwrap();
        for c in catchers {
            let mut cloned = c.clone();
            cloned.map_base(base.clone());
            list.push(cloned);
        }
    }

    #[napi]
    pub fn configure(&mut self, config: &Config) {
        self.config = Some(config.clone());
    }

    #[napi]
    pub fn ignite(&mut self) -> napi::Result<()> {
        self.phase = RocketPhase::Ignite;
        Ok(())
    }

    #[napi]
    pub fn launch(&mut self) -> napi::Result<()> {
        self.phase = RocketPhase::Orbit;
        Ok(())
    }

    #[napi]
    pub fn shutdown(&mut self) {
        self.phase = RocketPhase::Build;
    }

    #[napi]
    pub fn config(&self) -> Option<Config> {
        self.config.clone()
    }

    #[napi]
    pub fn routes(&self) -> Vec<Route> {
        self.routes.lock().unwrap().clone()
    }

    #[napi]
    pub fn catchers(&self) -> Vec<Catcher> {
        self.catchers.lock().unwrap().clone()
    }
}

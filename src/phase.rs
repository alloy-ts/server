//! NAPI bindings for Rocket's launch [`Phase`]s (`Build`, `Ignite`, `Orbit`).

use napi_derive::napi;

/// Rocket launch phase enumeration.
#[napi]
pub enum Phase {
  Build,
  Ignite,
  Orbit,
}

/// A representation of Rocket's `Build` phase state.
#[napi]
pub struct BuildPhase {
  pub route_count: u32,
  pub catcher_count: u32,
}

#[napi]
impl BuildPhase {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      route_count: 0,
      catcher_count: 0,
    }
  }
}

/// A representation of Rocket's `Ignite` phase state.
#[napi]
pub struct IgnitePhase {
  pub is_ignited: bool,
}

#[napi]
impl IgnitePhase {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self { is_ignited: true }
  }
}

/// A representation of Rocket's `Orbit` phase state.
#[napi]
pub struct OrbitPhase {
  pub is_orbiting: bool,
}

#[napi]
impl OrbitPhase {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self { is_orbiting: true }
  }
}

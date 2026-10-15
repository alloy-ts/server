//! # Rocket - Launch Phase NAPI Bindings
//!
//! Rocket instances progress through three launch phases:
//! Build -> Ignite -> Orbit.

use napi_derive::napi;

/// Rocket launch phases.
#[napi(string_enum)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Transient, in-progress build phase.
    Build,
    /// Post-build, pre-orbit verification and finalization phase.
    Ignite,
    /// Running web server actively serving requests.
    Orbit,
}

/// NAPI Phase marker object.
#[napi]
#[derive(Clone, Debug)]
pub struct PhaseState {
    pub current: Phase,
}

#[napi]
impl PhaseState {
    /// Constructs a Build phase state.
    #[napi]
    pub fn build() -> Self {
        Self {
            current: Phase::Build,
        }
    }

    /// Constructs an Ignite phase state.
    #[napi]
    pub fn ignite() -> Self {
        Self {
            current: Phase::Ignite,
        }
    }

    /// Constructs an Orbit phase state.
    #[napi]
    pub fn orbit() -> Self {
        Self {
            current: Phase::Orbit,
        }
    }

    /// Get current phase string representation.
    #[napi(js_name = "toString")]
    pub fn to_js_string(&self) -> String {
        format!("{:?}", self.current)
    }
}

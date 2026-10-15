//! NAPI bindings for Rocket's launch [`Phase`](rocket::phase::Phase).

use napi_derive::napi;

/// Rocket launch phase marker enum for JavaScript/TypeScript callers.
#[napi]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RocketPhase {
    Build,
    Ignite,
    Orbit,
}

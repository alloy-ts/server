//! NAPI bindings for Rocket's [`Fairing`] and [`Info`] (`rocket::fairing::Fairing`, `rocket::fairing::Info`).
//!
//! Fairings are Rocket's structured middleware allowing callbacks at ignite, liftoff,
//! request, response, and shutdown phases.

use napi_derive::napi;

/// Representation of Rocket's Fairing callback kinds (`rocket::fairing::Kind`).
///
/// Bitflags representing callback phases:
/// - `Ignite` (1): `on_ignite`
/// - `Liftoff` (2): `on_liftoff`
/// - `Request` (4): `on_request`
/// - `Response` (8): `on_response`
/// - `Shutdown` (16): `on_shutdown`
/// - `Singleton` (32): `Singleton`
#[derive(Clone, Debug)]
#[napi(object)]
pub struct FairingKind {
    pub ignite: bool,
    pub liftoff: bool,
    pub request: bool,
    pub response: bool,
    pub shutdown: bool,
    pub singleton: bool,
}

/// NAPI mirror of Rocket's `fairing::Info`.
#[derive(Clone, Debug)]
#[napi(object)]
pub struct FairingInfo {
    pub name: String,
    pub kind: FairingKind,
}

/// Helper function to construct FairingInfo.
#[napi]
pub fn create_fairing_info(name: String, kind: FairingKind) -> FairingInfo {
    FairingInfo { name, kind }
}

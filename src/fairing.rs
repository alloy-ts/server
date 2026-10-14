//! # NAPI bindings for Rocket's `Fairing` structures and middleware concepts.

use napi_derive::napi;

/// Bitset representing the kinds of callbacks a Fairing wishes to receive.
#[napi(object)]
pub struct FairingKind {
    pub ignite: bool,
    pub liftoff: bool,
    pub request: bool,
    pub response: bool,
    pub shutdown: bool,
    pub singleton: bool,
}

/// Information about a Fairing.
#[napi(object)]
pub struct FairingInfo {
    pub name: String,
    pub kind: FairingKind,
}

/// Helper function to create a [`FairingInfo`] instance.
#[napi]
pub fn create_fairing_info(name: String, kind: FairingKind) -> FairingInfo {
    FairingInfo { name, kind }
}

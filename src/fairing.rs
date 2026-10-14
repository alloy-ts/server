//! # Rocket - Fairing NAPI Bindings
//!
//! Fairings are Rocket's structured middleware callbacks at launch, liftoff,
//! request, response, and shutdown time.

use napi_derive::napi;

/// Callback kind bitset constants for Fairings.
#[napi(object)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct FairingKind {
    pub ignite: bool,
    pub liftoff: bool,
    pub request: bool,
    pub response: bool,
    pub shutdown: bool,
    pub singleton: bool,
}

/// Fairing Info metadata.
#[napi(object)]
#[derive(Clone, Debug)]
pub struct FairingInfo {
    pub name: String,
    pub kind: FairingKind,
}

/// Helper function to construct a custom FairingKind object.
#[napi]
pub fn create_fairing_kind(
    ignite: Option<bool>,
    liftoff: Option<bool>,
    request: Option<bool>,
    response: Option<bool>,
    shutdown: Option<bool>,
    singleton: Option<bool>,
) -> FairingKind {
    FairingKind {
        ignite: ignite.unwrap_or(false),
        liftoff: liftoff.unwrap_or(false),
        request: request.unwrap_or(false),
        response: response.unwrap_or(false),
        shutdown: shutdown.unwrap_or(false),
        singleton: singleton.unwrap_or(false),
    }
}

/// Predefined Kind: Request callback only.
#[napi]
pub fn request_fairing_kind() -> FairingKind {
    FairingKind {
        request: true,
        ..Default::default()
    }
}

/// Predefined Kind: Response callback only.
#[napi]
pub fn response_fairing_kind() -> FairingKind {
    FairingKind {
        response: true,
        ..Default::default()
    }
}

/// Ad-Hoc fairing metadata structure.
#[napi(object)]
#[derive(Clone, Debug)]
pub struct AdHocFairing {
    pub name: String,
    pub kind: FairingKind,
}

/// Create an Ad-Hoc fairing representation.
#[napi]
pub fn create_adhoc_fairing(name: String, kind: FairingKind) -> AdHocFairing {
    AdHocFairing { name, kind }
}

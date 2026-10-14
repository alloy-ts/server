//! NAPI bindings for Rocket's [`Fairing`] (`rocket::fairing::Fairing`).
//!
//! Fairings are Rocket's structured middleware, providing callbacks at
//! ignite, liftoff, request, response, and shutdown time.

use napi_derive::napi;

/// Bitset flags representing callback kinds for a Fairing.
#[napi]
pub const KIND_IGNITE: u32 = 1;
#[napi]
pub const KIND_LIFTOFF: u32 = 2;
#[napi]
pub const KIND_REQUEST: u32 = 4;
#[napi]
pub const KIND_RESPONSE: u32 = 8;
#[napi]
pub const KIND_SHUTDOWN: u32 = 16;
#[napi]
pub const KIND_SINGLETON: u32 = 32;

/// NAPI mirror of Rocket's `fairing::Info`.
#[napi(object)]
#[derive(Clone)]
pub struct FairingInfo {
    pub name: String,
    pub kind: u32,
}

/// NAPI wrapper for Rocket fairing configuration.
#[napi]
pub struct Fairing {
    name: String,
    kind: u32,
}

#[napi]
impl Fairing {
    #[napi(constructor)]
    pub fn new(name: String, kind: u32) -> Self {
        Self { name, kind }
    }

    /// Creates an ad-hoc fairing with a given name and callback kind mask.
    #[napi]
    pub fn ad_hoc(name: String, kind: Option<u32>) -> Self {
        let k = kind.unwrap_or(KIND_REQUEST | KIND_RESPONSE);
        Self { name, kind: k }
    }

    #[napi(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[napi(getter)]
    pub fn kind(&self) -> u32 {
        self.kind
    }

    #[napi]
    pub fn info(&self) -> FairingInfo {
        FairingInfo {
            name: self.name.clone(),
            kind: self.kind,
        }
    }
}

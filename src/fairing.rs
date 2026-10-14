//! # Rocket - Fairing NAPI Bindings
//!
//! This module implements NAPI bindings for Rocket's [`Fairing`] (structured middleware).
//!
//! Fairings run structured callbacks at key points in the application lifecycle:
//! - **Ignite (`on_ignite`)**: During ignition prior to liftoff; can modify configuration or attach fairings.
//! - **Liftoff (`on_liftoff`)**: Immediately after application launches; socket is open, before accepting connections.
//! - **Request (`on_request`)**: Just after a request is received, prior to routing.
//! - **Response (`on_response`)**: When a response is ready to be sent to the client.
//! - **Shutdown (`on_shutdown`)**: When graceful shutdown is triggered.

use napi_derive::napi;

/// Representation of Rocket fairing callback kinds.
#[napi]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FairingKind {
    Ignite = 1,
    Liftoff = 2,
    Request = 4,
    Response = 8,
    Shutdown = 16,
    Singleton = 32,
}

/// Metadata information about a Rocket Fairing.
#[napi(object)]
#[derive(Debug, Clone)]
pub struct FairingInfo {
    /// Arbitrary name identifying the fairing (logged during ignition).
    pub name: String,
    /// Bitmask or kind integer representing callbacks this fairing handles.
    pub kind: u32,
}

/// Constructor helper to create a `FairingInfo` object.
#[napi]
pub fn create_fairing_info(name: String, kind: u32) -> FairingInfo {
    FairingInfo { name, kind }
}

/// AdHoc fairing configuration representation.
#[napi(object)]
#[derive(Debug, Clone)]
pub struct AdHocFairing {
    pub name: String,
    pub kind: u32,
}

/// Create an AdHoc request/response fairing info object.
#[napi]
pub fn ad_hoc_fairing(name: String, kind: Option<u32>) -> AdHocFairing {
    let default_kind = (FairingKind::Request as u32) | (FairingKind::Response as u32);
    AdHocFairing {
        name,
        kind: kind.unwrap_or(default_kind),
    }
}

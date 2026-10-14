//! NAPI bindings for Rocket's [`Fairing`] (`rocket::fairing::Fairing`) and [`AdHoc`].

use napi_derive::napi;

/// Representation of Fairing callback kinds and metadata.
#[napi(object)]
pub struct FairingInfo {
    pub name: String,
    pub ignite: bool,
    pub liftoff: bool,
    pub request: bool,
    pub response: bool,
    pub shutdown: bool,
}

/// NAPI AdHoc fairing structure for Rocket structured middleware.
#[napi]
pub struct AdHoc {
    pub name: String,
    pub ignite: bool,
    pub liftoff: bool,
    pub request: bool,
    pub response: bool,
    pub shutdown: bool,
}

#[napi]
impl AdHoc {
    /// Creates an AdHoc fairing with the given name.
    #[napi(constructor)]
    pub fn new(name: String) -> Self {
        AdHoc {
            name,
            ignite: false,
            liftoff: false,
            request: false,
            response: false,
            shutdown: false,
        }
    }

    /// Creates an AdHoc ignite fairing.
    #[napi]
    pub fn on_ignite(name: String) -> Self {
        AdHoc {
            name,
            ignite: true,
            liftoff: false,
            request: false,
            response: false,
            shutdown: false,
        }
    }

    /// Creates an AdHoc liftoff fairing.
    #[napi]
    pub fn on_liftoff(name: String) -> Self {
        AdHoc {
            name,
            ignite: false,
            liftoff: true,
            request: false,
            response: false,
            shutdown: false,
        }
    }

    /// Creates an AdHoc request fairing.
    #[napi]
    pub fn on_request(name: String) -> Self {
        AdHoc {
            name,
            ignite: false,
            liftoff: false,
            request: true,
            response: false,
            shutdown: false,
        }
    }

    /// Creates an AdHoc response fairing.
    #[napi]
    pub fn on_response(name: String) -> Self {
        AdHoc {
            name,
            ignite: false,
            liftoff: false,
            request: false,
            response: true,
            shutdown: false,
        }
    }

    /// Creates an AdHoc shutdown fairing.
    #[napi]
    pub fn on_shutdown(name: String) -> Self {
        AdHoc {
            name,
            ignite: false,
            liftoff: false,
            request: false,
            response: false,
            shutdown: true,
        }
    }

    /// Returns the fairing info structure.
    #[napi]
    pub fn info(&self) -> FairingInfo {
        FairingInfo {
            name: self.name.clone(),
            ignite: self.ignite,
            liftoff: self.liftoff,
            request: self.request,
            response: self.response,
            shutdown: self.shutdown,
        }
    }
}

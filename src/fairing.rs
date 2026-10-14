//! NAPI bindings for Rocket's [`Fairing`](rocket::fairing::Fairing).

use napi_derive::napi;

#[napi(object)]
#[derive(Clone, Debug)]
pub struct FairingInfo {
    pub name: String,
    pub kind_ignite: Option<bool>,
    pub kind_liftoff: Option<bool>,
    pub kind_request: Option<bool>,
    pub kind_response: Option<bool>,
    pub kind_shutdown: Option<bool>,
    pub kind_singleton: Option<bool>,
}

#[napi]
#[derive(Clone)]
pub struct Fairing {
    pub name: String,
}

#[napi]
impl Fairing {
    #[napi(constructor)]
    pub fn new(name: String) -> Self {
        Self { name }
    }

    #[napi]
    pub fn info(&self) -> FairingInfo {
        FairingInfo {
            name: self.name.clone(),
            kind_ignite: Some(true),
            kind_liftoff: Some(true),
            kind_request: Some(true),
            kind_response: Some(true),
            kind_shutdown: Some(true),
            kind_singleton: Some(false),
        }
    }
}

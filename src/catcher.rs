//! NAPI bindings for Rocket's [`Catcher`] (`rocket::catcher::Catcher`).
//!
//! This module provides JavaScript / TypeScript bindings for Rocket's
//! error-catching route representation, allowing callers to inspect,
//! construct, and map catcher mount points programmatically.

use napi::bindgen_prelude::Status as NapiStatus;
use napi_derive::napi;

use rocket::catcher::{Catcher as RocketCatcher, Handler};
use rocket::http::uri::Origin;
use rocket::http::Status;
use rocket::{Request, Response};

/// Dummy catcher handler used when constructing NAPI Catcher instances.
#[derive(Clone)]
struct DummyCatcherHandler;

#[rocket::async_trait]
impl Handler for DummyCatcherHandler {
    async fn handle<'r>(&self, status: Status, _req: &'r Request<'_>) -> rocket::catcher::Result<'r> {
        let mut builder = Response::build();
        builder.status(status);
        Ok(builder.finalize())
    }
}

/// NAPI binding representing a Rocket error catcher (`rocket::catcher::Catcher`).
#[napi]
pub struct Catcher {
    name: Option<String>,
    base: String,
    code: Option<u16>,
}

#[napi]
impl Catcher {
    /// Creates a new catcher for the given status code (or `null`/`undefined` for default catcher).
    #[napi(constructor)]
    pub fn new(code: Option<u32>, name: Option<String>) -> napi::Result<Self> {
        let code = code.map(|c| c as u16);
        if let Some(c) = code {
            if c < 400 || c >= 600 {
                return Err(napi::Error::new(
                    NapiStatus::GenericFailure,
                    format!("status code {c} not in HTTP error range [400, 600)"),
                ));
            }
        }

        let mut rocket_catcher = RocketCatcher::new(code, DummyCatcherHandler);
        if let Some(ref n) = name {
            rocket_catcher.name = Some(n.clone().into());
        }

        Ok(Self {
            name,
            base: rocket_catcher.base.path().as_str().to_string(),
            code,
        })
    }

    /// Maps the base URI mount point of this catcher using a prefix string.
    #[napi]
    pub fn map_base(&mut self, new_base: String) -> napi::Result<()> {
        let rocket_catcher = RocketCatcher::new(self.code, DummyCatcherHandler);
        let mapped = rocket_catcher
            .map_base(|_| new_base.clone())
            .map_err(|e| napi::Error::new(NapiStatus::GenericFailure, e.to_string()))?;

        self.base = mapped.base.path().as_str().to_string();
        Ok(())
    }

    // Getters and Setters

    #[napi(getter)]
    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }

    #[napi(setter)]
    pub fn set_name(&mut self, value: Option<String>) {
        self.name = value;
    }

    #[napi(getter)]
    pub fn base(&self) -> String {
        self.base.clone()
    }

    #[napi(setter)]
    pub fn set_base(&mut self, value: String) -> napi::Result<()> {
        Origin::parse_owned(value.clone())
            .map_err(|e| napi::Error::new(NapiStatus::GenericFailure, e.to_string()))?;
        self.base = value;
        Ok(())
    }

    #[napi(getter)]
    pub fn code(&self) -> Option<u32> {
        self.code.map(|c| c as u32)
    }

    #[napi(setter)]
    pub fn set_code(&mut self, value: Option<u32>) -> napi::Result<()> {
        let code = value.map(|c| c as u16);
        if let Some(c) = code {
            if c < 400 || c >= 600 {
                return Err(napi::Error::new(
                    NapiStatus::GenericFailure,
                    format!("status code {c} not in HTTP error range [400, 600)"),
                ));
            }
        }
        self.code = code;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catcher_construction() {
        let catcher = Catcher::new(Some(404), Some("not_found".to_string())).unwrap();
        assert_eq!(catcher.code(), Some(404));
        assert_eq!(catcher.name(), Some("not_found".to_string()));
        assert_eq!(catcher.base(), "/");

        let default_catcher = Catcher::new(None, Some("default".to_string())).unwrap();
        assert_eq!(default_catcher.code(), None);
    }

    #[test]
    fn test_catcher_map_base() {
        let mut catcher = Catcher::new(Some(500), None).unwrap();
        assert_eq!(catcher.base(), "/");
        catcher.map_base("/api".to_string()).unwrap();
        assert_eq!(catcher.base(), "/api");
    }
}

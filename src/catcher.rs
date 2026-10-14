//! # NAPI bindings for Rocket's `Catcher` structure.

use napi_derive::napi;
use rocket::route::dummy_handler;
use rocket::Catcher as RocketCatcher;

#[napi]
pub struct Catcher {
    pub name: Option<String>,
    pub base: String,
    pub code: Option<u32>,
}

#[napi]
impl Catcher {
    /// Creates a catcher for the given status code (or default if None).
    #[napi(constructor)]
    pub fn new(code: Option<u32>, name: Option<String>) -> napi::Result<Self> {
        let code_u16 = match code {
            Some(c) => {
                if c >= 400 && c < 600 {
                    Some(c as u16)
                } else {
                    return Err(napi::Error::new(
                        napi::Status::InvalidArg,
                        format!("invalid HTTP status code for catcher: {c}"),
                    ));
                }
            }
            None => None,
        };

        let mut catcher = RocketCatcher::new(code_u16, dummy_handler);
        if let Some(n) = name {
            catcher.name = Some(n.into());
        }

        Ok(Catcher {
            name: catcher.name.map(|n| n.to_string()),
            base: catcher.base.path().to_string(),
            code: catcher.code.map(|c| c as u32),
        })
    }

    /// Maps the base URI of this catcher.
    #[napi]
    pub fn map_base(&mut self, base: String) -> napi::Result<()> {
        let dummy = RocketCatcher::new(self.code.map(|c| c as u16), dummy_handler);
        let mapped_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            dummy.map_base(|_| base.clone())
        }));

        match mapped_res {
            Ok(Ok(c)) => {
                self.base = c.base.path().to_string();
                Ok(())
            }
            _ => Err(napi::Error::new(
                napi::Status::InvalidArg,
                format!("invalid base URI for catcher: {base}"),
            )),
        }
    }
}

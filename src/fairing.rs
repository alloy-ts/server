//! NAPI bindings for Rocket's [`Fairing`](rocket::fairing::Fairing) middleware.
//!
//! Fairings allow for structured interposition at various points in the application lifetime:
//! ignition (`on_ignite`), liftoff (`on_liftoff`), request (`on_request`),
//! response (`on_response`), and shutdown (`on_shutdown`).

use std::sync::Arc;

use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
use napi::Unknown;
use napi_derive::napi;

use rocket::fairing::{Fairing as RocketFairing, Info as RocketInfo, Kind as RocketKind};
use rocket::{Build, Orbit, Rocket};

/// Information about a Fairing (name and callback kinds).
#[napi(object)]
pub struct FairingInfo {
    pub name: String,
    /// Bitset of callback kinds:
    /// Ignite = 1, Liftoff = 2, Request = 4, Response = 8, Shutdown = 16, Singleton = 32
    pub kind: u32,
}

#[napi]
pub const FAIRING_KIND_IGNITE: u32 = 1;
#[napi]
pub const FAIRING_KIND_LIFTOFF: u32 = 2;
#[napi]
pub const FAIRING_KIND_REQUEST: u32 = 4;
#[napi]
pub const FAIRING_KIND_RESPONSE: u32 = 8;
#[napi]
pub const FAIRING_KIND_SHUTDOWN: u32 = 16;
#[napi]
pub const FAIRING_KIND_SINGLETON: u32 = 32;

type IgniteCallbackTsfn = ThreadsafeFunction<(), Either<(), Promise<()>>, (), Status, false, false, 0>;
type LiftoffCallbackTsfn = ThreadsafeFunction<(), Either<(), Promise<()>>, (), Status, false, false, 0>;

#[allow(deprecated)]
fn to_js_function(handler: Unknown<'_>) -> napi::JsFunction {
    unsafe { std::mem::transmute_copy::<Unknown, napi::JsFunction>(&handler) }
}

/// An ad-hoc fairing created from a name and JS callbacks.
#[napi]
pub struct AdHoc {
    pub name: String,
    pub kind: u32,
    ignite_tsfn: Option<Arc<IgniteCallbackTsfn>>,
    liftoff_tsfn: Option<Arc<LiftoffCallbackTsfn>>,
}

#[napi]
impl AdHoc {
    /// Create an ad-hoc ignite fairing that executes a callback during ignition.
    #[napi]
    pub fn on_ignite(name: String, callback: Unknown<'_>) -> napi::Result<Self> {
        let tsfn: IgniteCallbackTsfn = to_js_function(callback).create_threadsafe_function::<
            (),
            (),
            Either<(), Promise<()>>,
            Status,
            _,
            false,
            false,
            0,
        >(|_| Ok(()))?;

        Ok(AdHoc {
            name,
            kind: FAIRING_KIND_IGNITE,
            ignite_tsfn: Some(Arc::new(tsfn)),
            liftoff_tsfn: None,
        })
    }

    /// Create an ad-hoc liftoff fairing that executes a callback immediately after launch.
    #[napi]
    pub fn on_liftoff(name: String, callback: Unknown<'_>) -> napi::Result<Self> {
        let tsfn: LiftoffCallbackTsfn = to_js_function(callback).create_threadsafe_function::<
            (),
            (),
            Either<(), Promise<()>>,
            Status,
            _,
            false,
            false,
            0,
        >(|_| Ok(()))?;

        Ok(AdHoc {
            name,
            kind: FAIRING_KIND_LIFTOFF,
            ignite_tsfn: None,
            liftoff_tsfn: Some(Arc::new(tsfn)),
        })
    }

    #[napi(getter)]
    pub fn info(&self) -> FairingInfo {
        FairingInfo {
            name: self.name.clone(),
            kind: self.kind,
        }
    }
}

#[rocket::async_trait]
impl RocketFairing for AdHoc {
    fn info(&self) -> RocketInfo {
        let mut kinds = Vec::new();
        if (self.kind & FAIRING_KIND_IGNITE) != 0 {
            kinds.push(RocketKind::Ignite);
        }
        if (self.kind & FAIRING_KIND_LIFTOFF) != 0 {
            kinds.push(RocketKind::Liftoff);
        }
        if (self.kind & FAIRING_KIND_REQUEST) != 0 {
            kinds.push(RocketKind::Request);
        }
        if (self.kind & FAIRING_KIND_RESPONSE) != 0 {
            kinds.push(RocketKind::Response);
        }
        if (self.kind & FAIRING_KIND_SHUTDOWN) != 0 {
            kinds.push(RocketKind::Shutdown);
        }
        if (self.kind & FAIRING_KIND_SINGLETON) != 0 {
            kinds.push(RocketKind::Singleton);
        }

        let mut final_kind = RocketKind::Ignite;
        if let Some(first) = kinds.first() {
            final_kind = *first;
            for k in kinds.iter().skip(1) {
                final_kind = final_kind | *k;
            }
        }

        RocketInfo {
            name: Box::leak(self.name.clone().into_boxed_str()),
            kind: final_kind,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> rocket::fairing::Result {
        if let Some(tsfn) = &self.ignite_tsfn {
            match tsfn.call_async(()).await {
                Ok(Either::A(_)) => Ok(rocket),
                Ok(Either::B(promise)) => match promise.await {
                    Ok(_) => Ok(rocket),
                    Err(_) => Err(rocket),
                },
                Err(_) => Err(rocket),
            }
        } else {
            Ok(rocket)
        }
    }

    async fn on_liftoff(&self, _rocket: &Rocket<Orbit>) {
        if let Some(tsfn) = &self.liftoff_tsfn {
            let res = tsfn.call_async(()).await;
            if let Ok(Either::B(promise)) = res {
                let _ = promise.await;
            }
        }
    }
}

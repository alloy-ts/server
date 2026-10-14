//! NAPI bindings for Rocket's [`Fairing`] (`rocket::fairing::Fairing`).

use std::sync::Arc;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
#[allow(deprecated)]
use napi::{JsFunction, Unknown};
use napi_derive::napi;

use rocket::fairing::{Fairing as RocketFairing, Info as RocketInfo, Kind as RocketKind};
use rocket::{Orbit, Rocket};

/// NAPI mirror of Rocket's Fairing `Kind` bitset flags.
#[napi(object)]
#[derive(Clone, Debug)]
pub struct FairingKind {
    pub ignite: Option<bool>,
    pub liftoff: Option<bool>,
    pub request: Option<bool>,
    pub response: Option<bool>,
    pub shutdown: Option<bool>,
    pub singleton: Option<bool>,
}

/// NAPI mirror of Rocket's Fairing `Info`.
#[napi(object)]
#[derive(Clone, Debug)]
pub struct FairingInfo {
    pub name: String,
    pub kind: FairingKind,
}

type OnLiftoffTsfn = ThreadsafeFunction<(), (), (), Status, true, false, 0>;

#[allow(deprecated)]
fn to_js_function(handler: Unknown) -> JsFunction {
    debug_assert_eq!(
        std::mem::size_of::<Unknown>(),
        std::mem::size_of::<JsFunction>()
    );
    unsafe { std::mem::transmute_copy::<Unknown, JsFunction>(&handler) }
}

/// NAPI wrapper for custom Rocket Fairings.
#[napi]
#[derive(Clone)]
pub struct CustomFairing {
    name: String,
    kind: FairingKind,
    on_liftoff_tsfn: Option<Arc<OnLiftoffTsfn>>,
}

#[napi]
impl CustomFairing {
    /// Create a new custom fairing with a name, kind, and optional `onLiftoff` callback.
    #[napi(constructor)]
    #[allow(deprecated)]
    pub fn new(
        name: String,
        kind: FairingKind,
        on_liftoff: Option<Unknown>,
    ) -> Result<Self> {
        let tsfn = if let Some(cb) = on_liftoff {
            let f = to_js_function(cb);
            let t: OnLiftoffTsfn = f.create_threadsafe_function::<(), (), (), Status, _, true, false, 0>(
                |_| Ok(()),
            )?;
            Some(Arc::new(t))
        } else {
            None
        };

        Ok(CustomFairing {
            name,
            kind,
            on_liftoff_tsfn: tsfn,
        })
    }
}

#[rocket::async_trait]
impl RocketFairing for CustomFairing {
    fn info(&self) -> RocketInfo {
        let mut kinds = Vec::new();
        if self.kind.ignite == Some(true) {
            kinds.push(RocketKind::Ignite);
        }
        if self.kind.liftoff == Some(true) {
            kinds.push(RocketKind::Liftoff);
        }
        if self.kind.request == Some(true) {
            kinds.push(RocketKind::Request);
        }
        if self.kind.response == Some(true) {
            kinds.push(RocketKind::Response);
        }
        if self.kind.shutdown == Some(true) {
            kinds.push(RocketKind::Shutdown);
        }
        if self.kind.singleton == Some(true) {
            kinds.push(RocketKind::Singleton);
        }

        let mut k = RocketKind::Liftoff;
        if let Some(first) = kinds.first() {
            k = *first;
            for next in &kinds[1..] {
                k = k | *next;
            }
        }

        let name_str: &'static str = Box::leak(self.name.clone().into_boxed_str());
        RocketInfo {
            name: name_str,
            kind: k,
        }
    }

    async fn on_liftoff(&self, _rocket: &Rocket<Orbit>) {
        if let Some(tsfn) = &self.on_liftoff_tsfn {
            let _: std::result::Result<(), napi::Error> = tsfn.call_async(Ok(())).await;
        }
    }
}

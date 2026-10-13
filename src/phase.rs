//! NAPI bindings for Service's three launch phases.
//!
//! Service encodes its build → start → active progression as a phantom type
//! parameter `P: Phase` on `Service<P>`. That type-state is a Rust-only
//! construct — the `Phase` trait is sealed and Service's `State` / `StateRef`
//! / `Stateful` machinery (and the `Building` / `Igniting` / `Active` state
//! structs) are `#[doc(hidden)]` / crate-private — so JavaScript sees a single
//! `Service` class whose current phase is tracked internally.
//!
//! * [`Phase`] is the JS-visible marker. It is a `#[napi]` class with three
//!   factory methods — [`Phase::build`], [`Phase::start`], [`Phase::active`] —
//!   mirroring Service's `Build` / `Ready` / `Active` marker types.
//! * [`ServiceStatus`] is the internal sum type (the NAPI analogue of Service's
//!   private `phase::State` enum) that owns the concrete `service::Service<P>`
//!   for whichever phase the JS `Service` currently occupies. `src/service.rs`
//!   takes the value out of its `Mutex` to transition phases
//!   (`mount` / `start` / `launch`) and puts the result back, so a `Service` is
//!   mutable only while in the `Build` variant — exactly like `Service<Build>`.

use service::{Build, Ready, Orbit as Active, Service as Svc};

/// Internal sum type: the concrete `service::Service<P>` for the phase the JS
/// `Service` object currently occupies.
///
/// This mirrors `service::phase::State` (which is `doc(hidden)` / crate-private)
/// but is fully accessible to this crate. `src/service.rs` takes the value out of
/// its `Mutex` to transition phases and puts a new one back.
pub(crate) enum ServiceStatus {
    /// `Service<Build>` — the only mutable phase.
    Build(Svc<Build>),
    /// `Service<Ready>` — finalised configuration, ready to launch.
    Ready(Svc<Ready>),
    /// `Service<Active>` — a running application.
    Active(Svc<Active>),
}

impl ServiceStatus {
    /// The phase of the currently-held service, as a JS-visible [`Phase`].
    pub(crate) fn phase(&self) -> Phase {
        match self {
            ServiceStatus::Build(_) => Phase::build(),
            ServiceStatus::Ready(_) => Phase::start(),
            ServiceStatus::Active(_) => Phase::active(),
        }
    }
}

/// The JS-visible launch-phase marker.
///
/// Instances are created by the three factory methods [`Phase::build`],
/// [`Phase::start`], and [`Phase::active`]; the `Service` object exposes its
/// current phase via [`crate::service::Service::phase`]. This is the NAPI analogue
/// of Service's sealed `Phase` trait and its `Build` / `Ready` / `Active` marker
/// types (re-exported from `service::phase` but not usable across the FFI
/// boundary, since the associated state structs are crate-private).
#[napi]
pub struct Phase {
    name: String,
}

#[napi]
impl Phase {
    /// The initial launch phase: application and service configuration.
    #[napi]
    pub fn build() -> Self {
        Phase { name: "build".into() }
    }

    /// The second launch phase: post-build but pre-live (finalised config).
    #[napi]
    pub fn start() -> Self {
        Phase { name: "start".into() }
    }

    /// The final launch phase: an active/live web service.
    #[napi]
    pub fn active() -> Self {
        Phase { name: "active".into() }
    }

    /// The phase's stable name (`"build"`, `"start"`, or `"active"`).
    #[napi(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

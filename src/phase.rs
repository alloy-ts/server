use napi_derive::napi;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PhaseKind {
    Build,
    Ignite,
    Orbit,
}

#[napi]
pub const PHASE_BUILD: &str = "Build";
#[napi]
pub const PHASE_IGNITE: &str = "Ignite";
#[napi]
pub const PHASE_ORBIT: &str = "Orbit";

#[napi]
pub struct Phase;

#[napi]
impl Phase {
    #[napi(js_name = "Build")]
    pub fn build() -> &'static str {
        "Build"
    }

    #[napi(js_name = "Ignite")]
    pub fn ignite() -> &'static str {
        "Ignite"
    }

    #[napi(js_name = "Orbit")]
    pub fn orbit() -> &'static str {
        "Orbit"
    }
}

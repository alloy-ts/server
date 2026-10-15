use napi_derive::napi;

#[napi]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Phase {
    Build,
    Ignite,
    Orbit,
}

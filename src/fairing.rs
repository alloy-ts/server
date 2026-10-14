use napi_derive::napi;

#[napi(object)]
#[derive(Debug, Clone, Default)]
pub struct FairingInfo {
  pub name: String,
  pub kind: u32,
}

#[napi]
#[derive(Debug, Clone, Copy)]
pub enum FairingKind {
  Ignite = 1,
  Liftoff = 2,
  Request = 4,
  Response = 8,
  Shutdown = 16,
  Singleton = 32,
}

#[napi]
pub struct Fairing {
  pub name: String,
  pub kind: u32,
}

#[napi]
impl Fairing {
  #[napi(constructor)]
  pub fn new(name: String, kind: u32) -> Self {
    Self { name, kind }
  }

  #[napi]
  pub fn info(&self) -> FairingInfo {
    FairingInfo {
      name: self.name.clone(),
      kind: self.kind,
    }
  }
}

#[napi]
pub struct AdHoc {
  pub name: String,
  pub kind: u32,
}

#[napi]
impl AdHoc {
  #[napi(factory)]
  pub fn on_ignite(name: String) -> Self {
    Self {
      name,
      kind: FairingKind::Ignite as u32,
    }
  }

  #[napi(factory)]
  pub fn on_liftoff(name: String) -> Self {
    Self {
      name,
      kind: FairingKind::Liftoff as u32,
    }
  }

  #[napi(factory)]
  pub fn on_request(name: String) -> Self {
    Self {
      name,
      kind: FairingKind::Request as u32,
    }
  }

  #[napi(factory)]
  pub fn on_response(name: String) -> Self {
    Self {
      name,
      kind: FairingKind::Response as u32,
    }
  }

  #[napi(factory)]
  pub fn on_shutdown(name: String) -> Self {
    Self {
      name,
      kind: FairingKind::Shutdown as u32,
    }
  }

  #[napi]
  pub fn info(&self) -> FairingInfo {
    FairingInfo {
      name: self.name.clone(),
      kind: self.kind,
    }
  }
}

#[napi(js_name = "createFairing")]
pub fn create_fairing(name: String, kind: u32) -> Fairing {
  Fairing::new(name, kind)
}

#[napi(js_name = "createAdHocFairing")]
pub fn create_adhoc_fairing(name: String, kind: u32) -> AdHoc {
  AdHoc { name, kind }
}

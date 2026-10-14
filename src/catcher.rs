use napi_derive::napi;
use rocket::catcher::Catcher as RocketCatcher;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::Response;
use rocket::catcher::BoxFuture;

fn dummy_catcher_handler<'r>(
    status: Status,
    _req: &'r Request<'_>,
) -> BoxFuture<'r> {
    Box::pin(async move {
        Ok(Response::build()
            .status(status)
            .finalize())
    })
}

/// NAPI binding for Rocket's `Catcher` struct.
#[napi]
pub struct Catcher {
  pub name: Option<String>,
  pub base: String,
  pub code: Option<u16>,
  pub(crate) inner: RocketCatcher,
}

#[napi]
impl Catcher {
  /// Creates a new `Catcher` for the given HTTP status code (or default/`null` status).
  #[napi(constructor)]
  pub fn new(code: Option<u16>, name: Option<String>) -> napi::Result<Self> {
    if let Some(c) = code {
      if !(400..600).contains(&c) {
        return Err(napi::Error::from_reason(
          "Status code must be in range [400, 600)",
        ));
      }
    }

    let mut catcher = RocketCatcher::new(code, dummy_catcher_handler);
    if let Some(ref n) = name {
      catcher.name = Some(n.clone().into());
    }

    Ok(Self {
      name,
      base: catcher.base.path().as_str().to_string(),
      code,
      inner: catcher,
    })
  }

  /// Maps the base mount point of this catcher.
  #[napi]
  pub fn map_base(&mut self, base: String) -> napi::Result<String> {
    let catcher = std::mem::replace(&mut self.inner, RocketCatcher::new(None, dummy_catcher_handler));
    match catcher.map_base(|_| base) {
      Ok(mapped) => {
        let new_base = mapped.base.path().as_str().to_string();
        self.base = new_base.clone();
        self.inner = mapped;
        Ok(new_base)
      }
      Err(e) => Err(napi::Error::from_reason(format!(
        "Invalid base URI: {}",
        e
      ))),
    }
  }

  /// Returns whether this catcher is a default catcher (no specific code).
  #[napi(getter)]
  pub fn is_default(&self) -> bool {
    self.code.is_none()
  }
}

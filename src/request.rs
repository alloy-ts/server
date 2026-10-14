//! NAPI bindings for Rocket's [`Request`] (`rocket::Request`).
//!
//! Exposes HTTP method, URI, client IP, content-type, accept header, and remote address.

use napi_derive::napi;

#[napi]
pub struct Request {
  method: String,
  uri: String,
  remote: Option<String>,
  real_ip: Option<String>,
  client_ip: Option<String>,
  content_type: Option<String>,
  accept: Option<String>,
  format: Option<String>,
}

#[napi]
impl Request {
  /// Create a new Request wrapper object.
  #[napi(constructor)]
  pub fn new(method: String, uri: String) -> Self {
    Request {
      method,
      uri,
      remote: None,
      real_ip: None,
      client_ip: None,
      content_type: None,
      accept: None,
      format: None,
    }
  }

  /// Returns the HTTP method.
  #[napi]
  pub fn method(&self) -> String {
    self.method.clone()
  }

  /// Sets the HTTP method.
  #[napi(js_name = "setMethod")]
  pub fn set_method(&mut self, method: String) {
    self.method = method;
  }

  /// Returns the URI string.
  #[napi]
  pub fn uri(&self) -> String {
    self.uri.clone()
  }

  /// Sets the URI.
  #[napi(js_name = "setUri")]
  pub fn set_uri(&mut self, uri: String) {
    self.uri = uri;
  }

  /// Returns remote socket address if present.
  #[napi]
  pub fn remote(&self) -> Option<String> {
    self.remote.clone()
  }

  /// Returns real IP address from headers.
  #[napi(js_name = "realIp")]
  pub fn real_ip(&self) -> Option<String> {
    self.real_ip.clone()
  }

  /// Returns client IP address.
  #[napi(js_name = "clientIp")]
  pub fn client_ip(&self) -> Option<String> {
    self.client_ip.clone()
  }

  /// Returns Content-Type header string.
  #[napi(js_name = "contentType")]
  pub fn content_type(&self) -> Option<String> {
    self.content_type.clone()
  }

  /// Returns Accept header string.
  #[napi]
  pub fn accept(&self) -> Option<String> {
    self.accept.clone()
  }

  /// Returns matched media type format string.
  #[napi]
  pub fn format(&self) -> Option<String> {
    self.format.clone()
  }
}

//! NAPI bindings that let Node.js drive a Rocket server.

use std::collections::HashMap;

use napi::bindgen_prelude::*;
use napi_derive::napi;

use rocket::http::Status as RocketStatus;
use rocket::tokio::io::AsyncReadExt;
use rocket::{Data, Request, Response};

// ---------------------------------------------------------------------------
// Main entrypoint and helpers
// ---------------------------------------------------------------------------

/// Main entrypoint function returning standard greeting.
#[napi]
pub fn main() -> String {
  "Hello, world!".to_string()
}

/// Convenience helper to create a Response object.
#[napi]
pub fn response(
  body: Option<String>,
  init: Option<crate::server::ResponseInit>,
) -> crate::server::Response {
  crate::server::Response::new(body, init)
}

/// Type-safe and URL-safe route URI constructor.
/// Replaces parameters like `:id`, `<id>`, or `<id..>` with encoded values.
#[napi]
pub fn uri(pattern: String, params: Option<HashMap<String, serde_json::Value>>) -> String {
  if let Some(params) = params {
    let mut result = pattern;
    for (key, value) in params {
      let val_str = match value {
        serde_json::Value::String(s) => s,
        serde_json::Value::Null => String::new(),
        other => serde_json::to_string(&other).unwrap_or_default(),
      };
      let encoded = val_str.replace(' ', "%20");
      result = result
        .replace(&format!(":{key}"), &encoded)
        .replace(&format!("<{key}..>"), &encoded)
        .replace(&format!("<{key}>"), &encoded);
    }
    result
  } else {
    pattern
  }
}

// ---------------------------------------------------------------------------
// JS <-> Rust translation
// ---------------------------------------------------------------------------

/// A request handed to a JS handler.
#[napi(object)]
pub struct RequestPayload {
  pub method: String,
  pub path: String,
  pub query: Option<String>,
  pub headers: Vec<JsHeader>,
  /// Path parameters extracted from the route pattern (e.g. `req.params.id`).
  pub params: Option<HashMap<String, String>>,
  /// Raw request body, or `undefined` when the request carries no body.
  pub body: Option<Buffer>,
}

#[napi(object)]
pub struct JsHeader {
  pub name: String,
  pub value: String,
}

/// A response returned from a JS handler.
#[napi(object)]
pub struct ResponsePayload {
  pub status: Option<u32>,
  pub headers: Option<Vec<JsHeader>>,
  pub body: Option<Buffer>,
}

/// Build a `RequestPayload` from a Rocket request.
pub(crate) async fn build_request_payload(
  req: &Request<'_>,
  data: Option<Data<'_>>,
) -> RequestPayload {
  let uri = req.uri();
  let method = req.method().as_str().to_string();
  let path = uri.path().to_string();
  let query = uri.query().map(|q| q.to_string());

  let headers: Vec<JsHeader> = req
    .headers()
    .iter()
    .map(|h| JsHeader {
      name: h.name.as_str().to_string(),
      value: h.value().to_string(),
    })
    .collect();

  let body = match data {
    Some(data) => {
      let mut stream = data.open(rocket::data::ByteUnit::max_value());
      let mut buf = Vec::new();
      let _ = stream.read_to_end(&mut buf).await;
      if buf.is_empty() {
        None
      } else {
        Some(Buffer::from(buf))
      }
    }
    None => None,
  };

  let pattern = req
    .route()
    .map(|r| r.uri.to_string())
    .unwrap_or_default();
  let params = crate::server::extract_params(&pattern, &path);

  RequestPayload {
    method,
    path,
    query,
    headers,
    params: if params.is_empty() {
      None
    } else {
      Some(params)
    },
    body,
  }
}

/// Build a Rocket `Response` from a JS handler's return value.
pub(crate) fn build_response(resp: ResponsePayload) -> Response<'static> {
  let status =
    RocketStatus::from_code(resp.status.unwrap_or(200) as u16).unwrap_or(RocketStatus::Ok);
  let mut builder = Response::build();
  builder.status(status);

  if let Some(headers) = resp.headers {
    for h in headers {
      builder.raw_header(h.name, h.value);
    }
  }

  if let Some(body) = resp.body {
    let bytes: Vec<u8> = body.to_vec();
    builder.sized_body(bytes.len(), std::io::Cursor::new(bytes));
  }

  builder.finalize()
}

/// Convenience mirror of `#[launch]`.
#[napi(js_name = "rocket_launch")]
pub async fn launch(config: Option<crate::rocket::RocketConfig>) -> Result<()> {
  crate::rocket::Rocket::new(config)?.launch().await
}

/// Mirror of `#[rocket::main]`'s inspectable form.
#[napi]
pub async fn rocket_main(config: Option<crate::rocket::RocketConfig>) -> Result<()> {
  crate::rocket::Rocket::new(config)?.launch().await
}

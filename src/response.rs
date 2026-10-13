//! NAPI bindings for the underlying framework's [`Response`] type and its
//! [`Builder`].
//!
//! The framework builds HTTP responses through a small builder API (see
//! [`service::response::Builder`]) on top of [`service::response::Response`].
//! This module mirrors that surface to TypeScript:
//!
//! * [`Response`] — a built response. Construct one with [`Response::build()`]
//!   (returns a [`Builder`]) or [`Response::to()`] (the `Responder` entry point
//!   that turns a string, a `Buffer`, or a `{ status, headers, body }` object
//!   into a `Response`, mirroring what `impl Responder` does in Rust).
//! * [`Builder`] — the chainable builder. Each mutating method returns `this`,
//!   so calls can be chained: `Response.build().status(200).header(..).finalise()`.
//!
//! Because NAPI objects must be `'static`, this binding only ever stores
//! *owned* bodies (`Cursor<Vec<u8>>`); borrowed/request-scoped bodies are never
//! carried across the FFI boundary.

use std::io::Cursor;

use napi::Unknown;
use napi::bindgen_prelude::{Buffer, Error, FromNapiValue, ValueType};
use napi::Status as NapiStatus;
use napi_derive::napi;
use service::http::{ContentType, Status};
use service::response::{Builder as ServiceBuilder, Response as ServiceResponse};

/// A built HTTP response, mirroring the underlying framework's `Response<'static>`.
///
/// Construct via [`Response::build()`] (returns a [`Builder`]) or
/// [`Response::to()`] (the `Responder` entry point). An empty response defaults
/// to `200 OK` with no body, matching the framework's `Response::new()`.
#[napi]
pub struct Response {
    inner: ServiceResponse<'static>,
}

impl Response {
    /// Build a NAPI [`Response`] from a raw framework `Response`.
    fn from_inner(inner: ServiceResponse<'static>) -> Response {
        Response { inner }
    }

    /// Build a `text/plain` response from a string body.
    fn from_text(s: String) -> Response {
        let mut b = ServiceResponse::build();
        b.status(Status::Ok);
        b.header(ContentType::Plain);
        b.sized_body(s.len(), Cursor::new(s.into_bytes()));
        Response::from_inner(b.finalise())
    }

    /// Build an `application/octet-stream` response from raw bytes.
    fn from_bytes(bytes: Vec<u8>) -> Response {
        let mut b = ServiceResponse::build();
        b.status(Status::Ok);
        b.header(ContentType::Binary);
        b.sized_body(bytes.len(), Cursor::new(bytes));
        Response::from_inner(b.finalise())
    }

    /// Build a response from a `{ status, headers, body }` init object.
    fn from_init(init: ResponseInit) -> Response {
        let mut b = ServiceResponse::build();
        if let Some(code) = init.status {
            if let Some(status) = Status::from_code(code as u16) {
                b.status(status);
            }
        }
        if let Some(headers) = init.headers {
            for (name, value) in headers {
                b.raw_header(name, value);
            }
        }
        if let Some(body) = init.body {
            b.sized_body(body.len(), Cursor::new(body.to_vec()));
        }
        Response::from_inner(b.finalise())
    }
}

/// The init object accepted by [`Response::to()`] for the object (responder) case:
/// `{ status?, headers?, body? }`.
///
/// `headers` is a list of `[name, value]` pairs (later entries with a
/// duplicate name are additional values for that header). `body` is a `Buffer`.
#[napi(object)]
pub struct ResponseInit {
    pub status: Option<u32>,
    pub headers: Option<Vec<(String, String)>>,
    pub body: Option<Buffer>,
}

#[napi]
impl Response {
    /// Create an empty `200 OK` response.
    #[napi(constructor)]
    pub fn new() -> Response {
        Response::from_inner(ServiceResponse::new())
    }

    /// Start building a response. Returns a [`Builder`] whose methods are
    /// chainable and end with [`Builder::finalise()`] (or [`Builder::ok()`]).
    #[napi]
    pub fn build() -> Builder {
        Builder {
            inner: ServiceResponse::build(),
        }
    }

    /// The `Responder` entry point: turn any supported value into a `Response`.
    ///
    /// Mirrors the framework's `Responder` trait for the common JS shapes:
    /// * `string` → `200`, `Content-Type: text/plain`, body = the string.
    /// * `Buffer` → `200`, `Content-Type: application/octet-stream`, body = bytes.
    /// * object `{ status?, headers?, body? }` → built per the fields above.
    /// * `null` / `undefined` → an empty `200 OK` response.
    ///
    /// This is the function the route layer should call to normalize a JS
    /// handler's return value into a framework `Response`.
    #[napi]
    pub fn to(value: Unknown) -> napi::Result<Response> {
        let env = value.env;
        match value.get_type()? {
            ValueType::Null | ValueType::Undefined => Ok(Response::from_inner(ServiceResponse::new())),
            ValueType::String => {
                let s = value.coerce_to_string()?.into_utf8()?.into_owned()?;
                Ok(Response::from_text(s))
            }
            ValueType::Buffer => {
                let buf = Buffer::from_napi_value(env, value.value.raw())?;
                Ok(Response::from_bytes(buf.to_vec()))
            }
            ValueType::Object => {
                let init = ResponseInit::from_napi_value(env, value.value.raw())?;
                Ok(Response::from_init(init))
            }
            other => Err(Error::new(
                NapiStatus::InvalidArg,
                format!("cannot build a Response from a value of type {other:?}"),
            )),
        }
    }

    /// The HTTP status code (e.g. `200` for the default `Ok`).
    #[napi]
    pub fn status(&self) -> u32 {
        self.inner.status().code as u32
    }

    /// Set the status code. Errors if `code` is not a valid HTTP status.
    #[napi]
    pub fn set_status(&mut self, code: u32) -> napi::Result<()> {
        let status = Status::from_code(code as u16).ok_or_else(|| {
            Error::new(NapiStatus::InvalidArg, format!("invalid HTTP status code: {code}"))
        })?;
        self.inner.set_status(status);
        Ok(())
    }

    /// The `Content-Type` header, if present (e.g. `"text/plain"`).
    #[napi]
    pub fn content_type(&self) -> Option<String> {
        self.inner.content_type().map(|c| c.to_string())
    }

    /// Set the `Content-Type` header by media type string (e.g. `"application/json"`).
    #[napi]
    pub fn set_content_type(&mut self, value: String) -> napi::Result<()> {
        let ct: ContentType = value
            .parse()
            .map_err(|_| Error::new(NapiStatus::InvalidArg, format!("invalid content type: {value}")))?;
        self.inner.set_header(ct);
        Ok(())
    }

    /// All headers as `[name, value]` pairs (multiple values per name appear
    /// as separate pairs, in order).
    #[napi]
    pub fn headers(&self) -> Vec<(String, String)> {
        self.inner
            .headers()
            .iter()
            .map(|h| (h.name.as_str().to_string(), h.value.to_string()))
            .collect()
    }

    /// The first value of header `name`, or `undefined` if absent.
    #[napi]
    pub fn header(&self, name: String) -> Option<String> {
        self.inner
            .headers()
            .get_one(&name)
            .map(|h| h.value.to_string())
    }

    /// All values of header `name` (empty if absent).
    #[napi]
    pub fn header_values(&self, name: String) -> Vec<String> {
        self.inner
            .headers()
            .get(&name)
            .map(|h| h.value.to_string())
            .collect()
    }

    /// Replace (or add) a single header `name` with `value`, removing any
    /// existing values for that name.
    #[napi]
    pub fn set_header(&mut self, name: String, value: String) {
        self.inner.set_raw_header(name, value);
    }

    /// Adjoin header `name` = `value`, keeping any existing values for that name.
    #[napi]
    pub fn add_header(&mut self, name: String, value: String) {
        self.inner.adjoin_raw_header(name, value);
    }

    /// Remove every header named `name`.
    #[napi]
    pub fn remove_header(&mut self, name: String) {
        self.inner.remove_header(&name);
    }

    /// Set a fixed-size body from a `Buffer` with an explicit `size` in bytes
    /// (or `undefined` to derive the size when written).
    #[napi]
    pub fn set_sized_body(&mut self, size: Option<u32>, body: Buffer) -> napi::Result<()> {
        self.inner.set_sized_body(size.map(|s| s as usize), Cursor::new(body.to_vec()));
        Ok(())
    }

    /// Set a streamed body from a `Buffer`.
    #[napi]
    pub fn set_streamed_body(&mut self, body: Buffer) -> napi::Result<()> {
        self.inner.set_streamed_body(Cursor::new(body.to_vec()));
        Ok(())
    }

    /// Replace this response's status/body/headers with those of `other` where
    /// `other` sets them (the framework's `Response::merge`).
    #[napi]
    pub fn merge(&mut self, other: Response) {
        self.inner.merge(other.inner);
    }

    /// Fill in this response's unset status/body/headers from `other` (the
    /// framework's `Response::join`).
    #[napi]
    pub fn join(&mut self, other: Response) {
        self.inner.join(other.inner);
    }
}

/// A higher-level builder for [`ServiceResponse`], mirroring the underlying framework's
/// `ServiceBuilder`. Methods return `this`, so they can be chained and finished with
/// [`ServiceBuilder::finalise()`] (or [`ServiceBuilder::ok()`]).
#[napi]
pub struct ServiceBuilder {
    inner: ServiceBuilder<'static>,
}

#[napi]
impl ServiceBuilder {
    /// Start a builder seeded from an existing [`ServiceResponse`].
    #[napi]
    pub fn from_response(response: ServiceResponse) -> ServiceBuilder {
        ServiceBuilder {
            inner: ServiceResponse::build_from(response.inner),
        }
    }

    /// Set the status code. Errors if `code` is not a valid HTTP status.
    #[napi]
    pub fn status(&mut self, code: u32) -> napi::Result<&mut Builder> {
        let status = Status::from_code(code as u16).ok_or_else(|| {
            Error::new(NapiStatus::InvalidArg, format!("invalid HTTP status code: {code}"))
        })?;
        self.inner.status(status);
        Ok(self)
    }

    /// Replace any existing header `name` with a single `value`.
    #[napi]
    pub fn header(&mut self, name: String, value: String) -> &mut Builder {
        self.inner.raw_header(name, value);
        self
    }

    /// Adjoin header `name` = `value`, keeping any existing values for `name`.
    #[napi]
    pub fn header_adjoin(&mut self, name: String, value: String) -> &mut Builder {
        self.inner.raw_header_adjoin(name, value);
        self
    }

    /// Set the `Content-Type` header by media type string (e.g. `"application/json"`).
    #[napi]
    pub fn content_type(&mut self, value: String) -> napi::Result<&mut Builder> {
        let ct: ContentType = value
            .parse()
            .map_err(|_| Error::new(NapiStatus::InvalidArg, format!("invalid content type: {value}")))?;
        self.inner.header(ct);
        Ok(self)
    }

    /// Set a fixed-size body from a `Buffer` with explicit `size` in bytes
    /// (or `undefined` to derive the size when written).
    #[napi]
    pub fn sized_body(&mut self, size: Option<u32>, body: Buffer) -> &mut Builder {
        self.inner.sized_body(size.map(|s| s as usize), Cursor::new(body.to_vec()));
        self
    }

    /// Set a streamed body from a `Buffer`.
    #[napi]
    pub fn streamed_body(&mut self, body: Buffer) -> &mut Builder {
        self.inner.streamed_body(Cursor::new(body.to_vec()));
        self
    }

    /// Set the maximum chunk size (bytes) used when streaming the body.
    #[napi]
    pub fn max_chunk_size(&mut self, size: u32) -> &mut Builder {
        self.inner.max_chunk_size(size as usize);
        self
    }

    /// Merge `other`'s status/body/headers into this builder (framework `merge`).
    #[napi]
    pub fn merge(&mut self, other: Response) -> &mut Builder {
        self.inner.merge(other.inner);
        self
    }

    /// Join `other`'s status/body/headers into this builder (framework `join`).
    #[napi]
    pub fn join(&mut self, other: Response) -> &mut Builder {
        self.inner.join(other.inner);
        self
    }

    /// Finish building and return the [`Response`]. The builder is cleared and
    /// must not be reused afterwards.
    #[napi]
    pub fn finalise(&mut self) -> Response {
        Response::from_inner(self.inner.finalise())
    }

    /// Like [`Builder::finalise()`], but returns the `Response` wrapped in `Ok`
    /// (matching the framework's `Builder::ok`, handy for handler returns).
    #[napi]
    pub fn ok(&mut self) -> napi::Result<Response> {
        Ok(Response::from_inner(self.inner.ok::<napi::Error>()?))
    }
}

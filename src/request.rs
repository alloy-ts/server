//! # NAPI bindings for the underlying framework's [`Request<'r>`].
//!
//! `Request<'r>` is the type the underlying framework hands to every
//! [`FromRequest`] guard and route handler: it carries the method, URI, headers,
//! cookies, client/remote addresses, the matched `Route`, and access to the live
//! service instance, but **not** the body (that is in `Data`). This module mirrors
//! that surface to TypeScript so a JS handler receives a `Request` object instead
//! of the older, narrower `RequestInput` snapshot.
//!
//! ## Lifetime / safety model
//!
//! `Request<'r>` is a borrowed type: the `&Request` only lives for the duration
//! of one handler invocation. NAPI cannot carry a borrowed Rust reference across
//! the FFI boundary (a `#[napi]` class must be `'static`), so this binding
//! stores the request as a raw pointer (`usize`) that is **valid only while the
//! JS handler is running** — which is exactly while the framework's `handle`
//! future is awaiting the JS callback. Concretely:
//!
//! * The object is built inside `JsHandler::handle` (see `crate::service`) from a
//!   live `&Request`, so every getter reads the real, live request.
//! * `body` and `params` are *owned* snapshots (the body lives in `Data`, the
//!   params are computed from the matched route pattern) and remain valid after
//!   the handler returns.
//! * **Do not** retain the `Request` object past the handler and call a getter
//!   on it later — the underlying request will have been freed. This matches the
//!   contract of every other FFI binding that exposes a borrowed Rust struct.
//!
//! ## `FromRequest` / `FromParam` mapping
//!
//! The framework's built-in request guards map onto the getters below:
//!
//! | Rust guard (`T: FromRequest`) | NAPI accessor                         |
//! |-------------------------------|---------------------------------------|
//! | `Method`                      | [`Request::method`]                   |
//! | `&Origin`                     | [`Request::uri`] / `path` / `query`   |
//! | `&Host`                       | [`Request::host`]                     |
//! | `&Route`                      | [`Request::route`]                    |
//! | `&CookieJar`                  | [`Request::cookies`]                  |
//! | `&Accept`                     | [`Request::accept`]                   |
//! | `&ContentType`                | [`Request::contentType`]              |
//! | `IpAddr`                      | [`Request::clientIp`]                 |
//! | `SocketAddr`                  | [`Request::remote`]                   |
//!
//! User-defined `FromRequest`/`FromParam` guards are implemented in JS by
//! reading these accessors (e.g. `req.headers().getOne("x-api-key")`,
//! `req.param(0)`). A future revision can expose `Request::guard(name)` to run a
//! registered Rust guard, but that is out of scope for this draft.

use std::collections::HashMap;

use napi_derive::napi;

use service::data::Limits;
use service::request::Request as ServiceRequest;

// ===========================================================================
// Supporting value types (all `#[napi]`-derived and `Send`)
// ===========================================================================

/// Mirror of the framework's `Host` type — the authority (domain[:port]) of the
/// request.
#[derive(Clone, Debug)]
#[napi(object)]
pub struct Host {
    /// The host domain (e.g. `"example.com"`).
    pub domain: String,
    /// The explicit port, if one was supplied (`None` means the scheme default).
    pub port: Option<u16>,
}

/// Mirror of a single framework `Cookie`.
#[derive(Clone, Debug)]
#[napi(object)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub path: Option<String>,
    pub domain: Option<String>,
    pub secure: Option<bool>,
    pub http_only: Option<bool>,
}

/// Read-only snapshot of the framework's `CookieJar`, the cookies attached to the
/// request. (The live jar supports internal mutability; from a handler it is
/// read-only, so this binding exposes a copied view.)
#[derive(Clone)]
#[napi]
pub struct CookieJar {
    cookies: Vec<Cookie>,
}

#[napi]
impl CookieJar {
    /// Number of cookies in the jar.
    #[napi(getter)]
    pub fn len(&self) -> u32 {
        self.cookies.len() as u32
    }

    /// `true` when the jar is empty.
    #[napi(getter)]
    pub fn is_empty(&self) -> bool {
        self.cookies.is_empty()
    }

    /// Fetch a cookie by name (first match). Mirrors `CookieJar::get`.
    #[napi]
    pub fn get(&self, name: String) -> Option<Cookie> {
        self.cookies
            .iter()
            .find(|c| c.name == name)
            .cloned()
    }

    /// Fetch a pending (just-added) cookie by name. In this snapshot view it is
    /// identical to [`CookieJar::get`]; the live distinction is not preserved.
    #[napi]
    pub fn get_pending(&self, name: String) -> Option<Cookie> {
        self.get(name)
    }

    /// `true` if a cookie with `name` exists.
    #[napi]
    pub fn contains(&self, name: String) -> bool {
        self.cookies.iter().any(|c| c.name == name)
    }

    /// All cookies as a list (mirrors iterating `CookieJar`).
    #[napi]
    pub fn to_list(&self) -> Vec<Cookie> {
        self.cookies.clone()
    }
}

/// Read-only view of the framework's `HeaderMap`. Header names are compared
/// case-insensitively (the framework uses `Uncased` keys), matching `HeaderMap`.
#[derive(Clone)]
#[napi]
pub struct RequestHeaders {
    headers: Vec<(String, String)>,
}

#[napi]
impl RequestHeaders {
    /// The first value for `name`, or `None`. Mirrors `HeaderMap::get_one`.
    #[napi]
    pub fn get_one(&self, name: String) -> Option<String> {
        let needle = name.to_ascii_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| k.to_ascii_lowercase() == needle)
            .map(|(_, v)| v.clone())
    }

    /// Every value for `name`. Mirrors `HeaderMap::get`.
    #[napi]
    pub fn get_all(&self, name: String) -> Vec<String> {
        let needle = name.to_ascii_lowercase();
        self.headers
            .iter()
            .filter(|(k, _)| k.to_ascii_lowercase() == needle)
            .map(|(_, v)| v.clone())
            .collect()
    }

    /// Number of header entries.
    #[napi(getter)]
    pub fn len(&self) -> u32 {
        self.headers.len() as u32
    }

    /// `true` when there are no headers.
    #[napi(getter)]
    pub fn is_empty(&self) -> bool {
        self.headers.is_empty()
    }

    /// `true` if a header named `name` exists.
    #[napi]
    pub fn contains(&self, name: String) -> bool {
        let needle = name.to_ascii_lowercase();
        self.headers
            .iter()
            .any(|(k, _)| k.to_ascii_lowercase() == needle)
    }

    /// All `(name, value)` pairs.
    #[napi]
    pub fn to_list(&self) -> Vec<(String, String)> {
        self.headers.clone()
    }
}

/// Read-only view of the framework's `Limits`, the configured data-size caps.
///
/// `Limits` is not publicly iterable, so this binding exposes named lookups via
/// [`RequestLimits::get`]. The pointer is valid only while the owning `Request`
/// is alive (i.e. during the handler call), consistent with the module caveat.
#[derive(Clone)]
#[napi]
pub struct RequestLimits {
    inner: usize, // *const service::data::Limits<'static>
}

impl RequestLimits {
    fn limits(&self) -> &Limits {
        unsafe { &*(self.inner as *const Limits) }
    }
}

#[napi]
impl RequestLimits {
    /// The limit for `name`, formatted as the framework's byte-unit string (e.g.
    /// `"32KiB"`), or `None` if no limit is configured for that name.
    #[napi]
    pub fn get(&self, name: String) -> Option<String> {
        self.limits().get(&name).map(|b| b.to_string())
    }
}

/// Mirror of the currently matched `Route`, if any. `None` before
/// routing has commenced (e.g. during request fairing callbacks).
#[derive(Clone, Debug)]
#[napi(object)]
pub struct RouteRef {
    /// The route's name, if one was assigned.
    pub name: Option<String>,
    /// The route's method.
    pub method: String,
    /// The route's URI (base + pattern).
    pub uri: String,
    /// The route's rank (lower = higher precedence).
    pub rank: i32,
}

// ===========================================================================
// Request — the NAPI mirror of the framework's `Request<'r>`
// ===========================================================================

/// NAPI mirror of the framework's `Request<'r>`.
///
/// Holds a raw pointer to the live request (valid only during the JS
/// handler call — see the module docs) plus the owned `body`/`params` snapshots.
/// Every getter other than `body`/`params` reads the live request.
#[napi]
pub struct Request {
    inner: usize, // *const service::request::Request<'static>
    /// The (possibly truncated) request body — owned snapshot; see module docs.
    body: Option<Vec<u8>>,
    /// The matched route's path parameters, keyed by name — owned snapshot.
    params: HashMap<String, String>,
}

impl Request {
    /// Build a `Request` NAPI object from a live `&Request`.
    ///
    /// `body` and `params` are supplied by the caller (the body comes from
    /// `Data`; the params are computed from the matched route pattern by
    /// `crate::service::extract_params`). The live request pointer is stored so
    /// the getters below read the real request.
    pub(crate) fn build(
        req: &ServiceRequest<'_>,
        body: Option<Vec<u8>>,
        params: HashMap<String, String>,
    ) -> Request {
        Request {
            inner: (req as *const ServiceRequest<'_>) as usize,
            body,
            params,
        }
    }

    /// Recover the live request reference. Unsafe by construction: the pointer is
    /// only valid while the originating handler call is in flight.
    #[inline]
    fn req(&self) -> &ServiceRequest<'static> {
        unsafe { &*(self.inner as *const ServiceRequest<'static>) }
    }
}

#[napi]
impl Request {
    /// The HTTP method (e.g. `"GET"`). Mirrors the framework's `Request::method`.
    #[napi(getter)]
    pub fn method(&self) -> String {
        self.req().method().as_str().to_string()
    }

    /// The full request URI. Mirrors the framework's `Request::uri`.
    #[napi(getter)]
    pub fn uri(&self) -> String {
        self.req().uri().to_string()
    }

    /// The request path (the URI without query). Mirrors `uri().path()`.
    #[napi(getter)]
    pub fn path(&self) -> String {
        self.req().uri().path().to_string()
    }

    /// The raw query string (without the leading `?`), or `""` if absent.
    #[napi(getter)]
    pub fn query(&self) -> String {
        self.req()
            .uri()
            .query()
            .map(|q| q.to_string())
            .unwrap_or_default()
    }

    /// The authority (`host[:port]`) from the request, if present. Mirrors the
    /// `:authority` pseudo-header / `HOST` header that [`Request::host`] exposes.
    #[napi(getter)]
    pub fn authority(&self) -> Option<String> {
        self.req().host().map(|h| match h.port() {
            Some(p) => format!("{}:{}", h.domain(), p),
            None => h.domain().to_string(),
        })
    }

    /// The [`Host`] of the request, if one was identified. Mirrors
    /// the framework's `Request::host`.
    #[napi(getter)]
    pub fn host(&self) -> Option<Host> {
        self.req().host().map(|h| Host {
            domain: h.domain().to_string(),
            port: h.port(),
        })
    }

    /// The remote (peer) socket address, or `None` if unknown. Mirrors
    /// the framework's `Request::remote`.
    #[napi(getter)]
    pub fn remote(&self) -> Option<String> {
        self.req().remote().map(|s| s.to_string())
    }

    /// The client IP from the configured `ip_header` (default `X-Real-IP`), if
    /// present and valid. Mirrors the framework's `Request::real_ip`.
    #[napi(getter)]
    pub fn real_ip(&self) -> Option<String> {
        self.req().real_ip().map(|ip| ip.to_string())
    }

    /// The client IP: the `ip_header` value when present, else the remote
    /// address. Mirrors the framework's `Request::client_ip`.
    #[napi(getter)]
    pub fn client_ip(&self) -> Option<String> {
        self.req().client_ip().map(|ip| ip.to_string())
    }

    /// Headers as a `name -> value` map (first value per name). Retained for
    /// backwards compatibility with the old `RequestInput` shape. Prefer
    /// [`Request::header_map`] for multi-value headers.
    #[napi(getter)]
    pub fn headers(&self) -> HashMap<String, String> {
        self.req()
            .headers()
            .iter()
            .map(|h| (h.name.as_str().to_string(), h.value.to_string()))
            .collect()
    }

    /// Headers as a rich [`RequestHeaders`] view (multi-value aware).
    #[napi(getter)]
    pub fn header_map(&self) -> RequestHeaders {
        RequestHeaders {
            headers: self
                .req()
                .headers()
                .iter()
                .map(|h| (h.name.as_str().to_string(), h.value.to_string()))
                .collect(),
        }
    }

    /// The request cookies as a read-only [`CookieJar`] view. Mirrors
    /// the framework's `Request::cookies`.
    #[napi(getter)]
    pub fn cookies(&self) -> CookieJar {
        CookieJar {
            cookies: self
                .req()
                .cookies()
                .iter()
                .map(|c| Cookie {
                    name: c.name().to_string(),
                    value: c.value().to_string(),
                    path: c.path().map(|s| s.to_string()),
                    domain: c.domain().map(|s| s.to_string()),
                    secure: c.secure(),
                    http_only: c.http_only(),
                })
                .collect(),
        }
    }

    /// The `Content-Type` header, or `None`. Mirrors
    /// the framework's `Request::content_type`.
    #[napi(getter)]
    pub fn content_type(&self) -> Option<String> {
        self.req().content_type().map(|c| c.to_string())
    }

    /// The `Accept` header, or `None`. Mirrors
    /// the framework's `Request::accept`.
    #[napi(getter)]
    pub fn accept(&self) -> Option<String> {
        self.req().accept().map(|a| a.to_string())
    }

    /// The negotiated media-type "format" of the request. Mirrors
    /// the framework's `Request::format`.
    #[napi(getter)]
    pub fn format(&self) -> Option<String> {
        self.req().format().map(|m| m.to_string())
    }

    /// The matched [`RouteRef`], or `None` before routing has commenced.
    /// Mirrors the framework's `Request::route`.
    #[napi(getter)]
    pub fn route(&self) -> Option<RouteRef> {
        self.req().route().map(|r| RouteRef {
            name: r.name.as_ref().map(|c| c.to_string()),
            method: r.method.as_str().to_string(),
            uri: r.uri.to_string(),
            rank: r.rank as i32,
        })
    }

    /// The configured application data limits. Mirrors
    /// the framework's `Request::limits`.
    #[napi(getter)]
    pub fn limits(&self) -> RequestLimits {
        RequestLimits {
            inner: (self.req().limits() as *const Limits) as usize,
        }
    }

    /// The 0-indexed `n`th non-empty path segment after the mount point,
    /// parsed as a string. Mirrors the framework's `Request::param`
    /// (`FromParam` for `&str`). Returns `None` when `n` is out of range.
    #[napi]
    pub fn param(&self, n: u32) -> Option<String> {
        self.req()
            .param::<&str>(n as usize)
            .and_then(|r| r.ok().map(|s| s.to_string()))
    }

    /// The (possibly truncated) request body — owned snapshot; see module docs.
    #[napi(getter)]
    pub fn body(&self) -> Option<Vec<u8>> {
        self.body.clone()
    }

    /// The matched route's path parameters, keyed by name — owned snapshot.
    #[napi(getter)]
    pub fn params(&self) -> HashMap<String, String> {
        self.params.clone()
    }

    /// Overwrite the owned body snapshot (local to this JS object only; does not
    /// affect the live `Data` stream). Provided so handlers can normalize the
    /// body they received.
    #[napi(setter)]
    pub fn set_body(&mut self, body: Option<Vec<u8>>) {
        self.body = body;
    }

    /// Overwrite the owned params snapshot (local to this JS object only).
    #[napi(setter)]
    pub fn set_params(&mut self, params: HashMap<String, String>) {
        self.params = params;
    }
}

// Compile-time reminder that `Request` is only safe to read while the live
// framework request is alive. (No `Drop` is needed: the raw pointer is never owned.)
const _: () = {
    // `Request` is `Send` (the inner pointer is stored as `usize`), which is
    // what `ThreadsafeFunction<Request, ..>` requires.
};

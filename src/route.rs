//! NAPI bindings for Rocket's [`Route`] (`rocket::Route`).
//!
//! Exposes Rocket's `Route` metadata, rank calculation, and `map_base` operations
//! to Node.js / TypeScript.

use napi::bindgen_prelude::*;
#[allow(deprecated)]
use napi::{JsFunction, Unknown, ValueType};
use napi_derive::napi;

use rocket::http::{MediaType, Method};
use rocket::route::dummy_handler;
use rocket::Route as RocketRoute;

#[allow(deprecated)]
fn to_js_function(u: Unknown) -> JsFunction {
    debug_assert_eq!(
        std::mem::size_of::<Unknown>(),
        std::mem::size_of::<JsFunction>()
    );
    unsafe { std::mem::transmute_copy::<Unknown, JsFunction>(&u) }
}

/// A request handling route metadata structure.
#[napi]
#[derive(Clone, Debug)]
pub struct Route {
    pub name: Option<String>,
    pub method: String,
    pub uri: String,
    pub rank: i64,
    pub format: Option<String>,
}

#[napi]
impl Route {
    /// Creates a new route with the given method, path, and computed default rank.
    #[napi(constructor)]
    #[allow(deprecated)]
    pub fn new(method: String, uri: String, name: Option<String>, format: Option<String>) -> napi::Result<Self> {
        let m: Method = method
            .parse()
            .map_err(|_| napi::Error::new(Status::GenericFailure, format!("invalid method: {method}")))?;
        let mut route = RocketRoute::new(m, &uri, dummy_handler);
        if let Some(n) = name {
            route.name = Some(n.into());
        }
        if let Some(f) = format {
            route.format = MediaType::parse_flexible(&f);
        }
        Ok(Self {
            name: route.name.map(|n| n.to_string()),
            method: route.method.as_str().to_string(),
            uri: route.uri.to_string(),
            rank: route.rank as i64,
            format: route.format.map(|f| f.to_string()),
        })
    }

    /// Creates a new route with an explicit rank (or None for default rank).
    #[napi(factory)]
    #[allow(deprecated)]
    pub fn ranked(
        rank: Option<i64>,
        method: String,
        uri: String,
        name: Option<String>,
        format: Option<String>,
    ) -> napi::Result<Self> {
        let m: Method = method
            .parse()
            .map_err(|_| napi::Error::new(Status::GenericFailure, format!("invalid method: {method}")))?;
        let r: Option<isize> = rank.map(|r| r as isize);
        let mut route = RocketRoute::ranked(r, m, &uri, dummy_handler);
        if let Some(n) = name {
            route.name = Some(n.into());
        }
        if let Some(f) = format {
            route.format = MediaType::parse_flexible(&f);
        }
        Ok(Self {
            name: route.name.map(|n| n.to_string()),
            method: route.method.as_str().to_string(),
            uri: route.uri.to_string(),
            rank: route.rank as i64,
            format: route.format.map(|f| f.to_string()),
        })
    }

    /// Maps the base of this route using a string or a callback function.
    #[napi]
    #[allow(deprecated)]
    pub fn map_base(&self, base_or_fn: Unknown) -> napi::Result<Self> {
        let m: Method = self
            .method
            .parse()
            .map_err(|_| napi::Error::new(Status::GenericFailure, format!("invalid method: {}", self.method)))?;
        let mut route = RocketRoute::ranked(Some(self.rank as isize), m, &self.uri, dummy_handler);
        if let Some(n) = &self.name {
            route.name = Some(n.clone().into());
        }
        if let Some(f) = &self.format {
            route.format = MediaType::parse_flexible(f);
        }

        let mapped = if base_or_fn.get_type()? == ValueType::Function {
            let js_fn = to_js_function(base_or_fn);
            let current_base = route.uri.base().to_string();
            let new_base: String = js_fn.call1(current_base)?;
            route.map_base(|_| new_base)
        } else if base_or_fn.get_type()? == ValueType::String {
            let new_base: String = unsafe { base_or_fn.cast()? };
            route.map_base(|_| new_base)
        } else {
            return Err(napi::Error::new(
                Status::InvalidArg,
                "expected string or function for map_base",
            ));
        }
        .map_err(|e| napi::Error::new(Status::GenericFailure, e.to_string()))?;

        Ok(Self {
            name: mapped.name.map(|n| n.to_string()),
            method: mapped.method.as_str().to_string(),
            uri: mapped.uri.to_string(),
            rank: mapped.rank as i64,
            format: mapped.format.map(|f| f.to_string()),
        })
    }

    /// Formatted string description of the route.
    #[napi]
    pub fn to_string(&self) -> String {
        format!("{} {} (rank: {})", self.method, self.uri, self.rank)
    }
}

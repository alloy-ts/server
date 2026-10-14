//! NAPI exports for App class and main entry.

use napi_derive::napi;
use napi::bindgen_prelude::{Object, JsObjectValue};
use napi::{Unknown, Result, ValueType};
use crate::server::{serve, Server, ServeOptions, file, StaticFile};

#[napi]
pub fn main() -> String {
    "Hello, world!".to_string()
}

/// NAPI `Alloy` namespace provider.
#[napi]
pub struct Alloy;

#[napi]
impl Alloy {
    /// `Alloy.serve(options)` accepts a single options object containing
    /// `routes`, `fetch`, `port`, `hostname`, `unix`.
    #[napi]
    pub fn serve(options: Unknown<'_>) -> Result<Server> {
        if options.get_type()? != ValueType::Object {
            return serve(None, None, None);
        }
        let obj: Object = unsafe { options.cast()? };
        let routes: Option<Unknown<'_>> = obj.get_named_property("routes")?;
        let fetch: Option<Unknown<'_>> = obj.get_named_property("fetch")?;
        let port: Option<u32> = obj.get_named_property("port")?;
        let hostname: Option<String> = obj.get_named_property("hostname")?;
        let unix: Option<String> = obj.get_named_property("unix")?;

        serve(routes, fetch, Some(ServeOptions { port, hostname, unix }))
    }

    #[napi]
    pub fn file(path: String) -> StaticFile {
        file(path)
    }
}

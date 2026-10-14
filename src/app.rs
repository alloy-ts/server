//! NAPI bindings for the `App` express/method-chaining application builder.

use std::collections::HashMap;

use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::server::{serve, ServeOptions, Server};

/// Application builder re-expressing Rocket's macros at runtime.
#[napi]
pub struct App {
    routes_table: HashMap<String, Unknown<'static>>,
    server_instance: Option<Server>,
}

#[napi]
impl App {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            routes_table: HashMap::new(),
            server_instance: None,
        }
    }

    #[napi]
    pub fn get(&mut self, env: Env, path: String, handler: Unknown) -> napi::Result<()> {
        self.route(env, "GET".to_string(), path, handler)
    }

    #[napi]
    pub fn post(&mut self, env: Env, path: String, handler: Unknown) -> napi::Result<()> {
        self.route(env, "POST".to_string(), path, handler)
    }

    #[napi]
    pub fn put(&mut self, env: Env, path: String, handler: Unknown) -> napi::Result<()> {
        self.route(env, "PUT".to_string(), path, handler)
    }

    #[napi]
    pub fn delete(&mut self, env: Env, path: String, handler: Unknown) -> napi::Result<()> {
        self.route(env, "DELETE".to_string(), path, handler)
    }

    #[napi]
    pub fn patch(&mut self, env: Env, path: String, handler: Unknown) -> napi::Result<()> {
        self.route(env, "PATCH".to_string(), path, handler)
    }

    #[napi]
    pub fn head(&mut self, env: Env, path: String, handler: Unknown) -> napi::Result<()> {
        self.route(env, "HEAD".to_string(), path, handler)
    }

    #[napi]
    pub fn options(&mut self, env: Env, path: String, handler: Unknown) -> napi::Result<()> {
        self.route(env, "OPTIONS".to_string(), path, handler)
    }

    #[napi]
    pub fn route(
        &mut self,
        env: Env,
        method: String,
        path: String,
        handler: Unknown,
    ) -> napi::Result<()> {
        let m = method.to_ascii_uppercase();
        let existing = self.routes_table.get(&path);

        if let Some(existing_val) = existing {
            if existing_val.get_type()? == ValueType::Object {
                let mut map_obj: Object = unsafe { existing_val.cast()? };
                map_obj.set_named_property(&m, handler)?;
            } else {
                let mut map_obj = env.create_object()?;
                map_obj.set_named_property("GET", existing_val.clone())?;
                map_obj.set_named_property(&m, handler)?;
                self.routes_table.insert(path, map_obj.into_unknown());
            }
        } else {
            let mut map_obj = env.create_object()?;
            map_obj.set_named_property(&m, handler)?;
            self.routes_table.insert(path, map_obj.into_unknown());
        }

        Ok(())
    }

    #[napi]
    pub fn mount(&mut self, base: String, routes: Object) -> napi::Result<()> {
        let prefix = if base == "/" { "" } else { &base };
        let names = routes.get_property_names()?;
        let len = names.get_array_length_unchecked()?;
        for i in 0..len {
            let path: String = names.get_element(i)?;
            let val: Unknown = routes.get_named_property(&path)?;
            let full_path = if path.starts_with('/') {
                format!("{prefix}{path}")
            } else {
                format!("{prefix}/{path}")
            };
            self.routes_table.insert(full_path, val);
        }
        Ok(())
    }

    #[napi]
    pub fn launch(&mut self, env: Env, options: Option<ServeOptions>) -> napi::Result<Server> {
        let mut routes_obj = env.create_object()?;
        for (path, val) in &self.routes_table {
            routes_obj.set_named_property(path, val.clone())?;
        }

        let s = serve(Some(routes_obj.into_unknown()), None, options)?;
        self.server_instance = Some(Server {
            url: s.url.clone(),
            port: s.port,
        });
        Ok(s)
    }

    #[napi]
    pub fn stop(&self) -> napi::Result<()> {
        if let Some(s) = &self.server_instance {
            s.stop()?;
        }
        Ok(())
    }
}

/// Simple top-level helper function returning `"Hello, world!"`.
#[napi]
pub fn main() -> String {
    "Hello, world!".to_string()
}

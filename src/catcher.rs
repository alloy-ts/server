//! NAPI bindings for Rocket's [`Catcher`](rocket::Catcher).

use napi_derive::napi;

use rocket::catcher::{BoxFuture, Catcher as RocketCatcher};
use rocket::http::Status as RocketStatus;
use rocket::response::Responder;
use rocket::Request;

fn dummy_catcher_handler<'r>(s: RocketStatus, req: &'r Request<'_>) -> BoxFuture<'r> {
    let res = (s, format!("{s}: {}", req.uri()));
    Box::pin(async move { res.respond_to(req) })
}

#[napi]
#[derive(Clone)]
pub struct Catcher {
    pub name: Option<String>,
    pub base: String,
    pub code: Option<u32>,
}

#[napi]
impl Catcher {
    #[napi(constructor)]
    pub fn new(code: Option<u32>, name: Option<String>) -> Self {
        let code_u16 = code.map(|c| c as u16);
        let rocket_catcher = RocketCatcher::new(code_u16, dummy_catcher_handler);

        Self {
            name: name.or_else(|| rocket_catcher.name.map(|n| n.to_string())),
            base: rocket_catcher.base.path().to_string(),
            code,
        }
    }

    #[napi]
    pub fn map_base(&mut self, base_prefix: String) {
        if !base_prefix.is_empty() && base_prefix != "/" {
            let clean_prefix = base_prefix.trim_end_matches('/');
            self.base = format!("{clean_prefix}{}", self.base);
        }
    }
}

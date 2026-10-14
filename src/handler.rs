#[allow(deprecated)]
use napi::{Env, JsFunction, Unknown};
use napi_derive::napi;

/// Convert a `napi::Unknown` into a `napi::JsFunction`.
#[allow(deprecated)]
fn to_js_function(handler: Unknown) -> JsFunction {
    debug_assert_eq!(
        std::mem::size_of::<Unknown>(),
        std::mem::size_of::<JsFunction>()
    );
    unsafe { std::mem::transmute_copy::<Unknown, JsFunction>(&handler) }
}

/// NAPI wrapper around Rocket route handlers.
#[napi(js_name = "Handler")]
#[derive(Clone, Default)]
pub struct Handler {
    pub valid: bool,
}

#[napi]
impl Handler {
    /// Constructs a Handler from a JavaScript request callback function or unknown value.
    #[napi(constructor)]
    pub fn new(
        env: Env,
        #[napi(ts_arg_type = "((req: any) => any) | null | undefined")] callback: Option<
            Unknown<'_>,
        >,
    ) -> napi::Result<Self> {
        if let Some(cb) = callback {
            let factory_unknown: Unknown =
                env.run_script("handler => async (req) => handler(req)")?;
            let factory = to_js_function(factory_unknown);
            let handler_func = to_js_function(cb);
            let _async_handler_unknown = factory.call(None, &[handler_func])?;
            Ok(Self { valid: true })
        } else {
            Ok(Self { valid: false })
        }
    }

    /// Helper to test if a Handler callback is valid and defined.
    #[napi(js_name = "isValid")]
    pub fn is_valid(&self) -> bool {
        self.valid
    }
}

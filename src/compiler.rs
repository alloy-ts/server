//! # Compiler NAPI Bindings
//!
//! NAPI bindings for Compiler execution using OXC.

use std::path::PathBuf;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use napi_derive::napi;

#[napi]
#[derive(Default)]
pub struct Compiler;

#[napi]
impl Compiler {
  #[napi(constructor)]
  pub fn new() -> Self {
    Compiler
  }

  /// Executes compilation or transformation on the source text using OXC parser.
  #[napi]
  pub fn execute(
    &mut self,
    source_text: String,
    source_type: Option<String>,
    source_path: Option<String>,
  ) -> napi::Result<String> {
    let path = PathBuf::from(source_path.unwrap_or_else(|| "index.ts".to_string()));
    let st_str = source_type.unwrap_or_else(|| "ts".to_string());

    if source_text.is_empty() {
      return Ok(String::new());
    }

    let allocator = Allocator::default();
    let source_type = SourceType::from_path(&path).unwrap_or_else(|_| match st_str.as_str() {
      "jsx" => SourceType::jsx(),
      "tsx" => SourceType::tsx(),
      "js" | "mjs" | "cjs" => SourceType::mjs(),
      _ => SourceType::ts(),
    });

    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    if !ret.diagnostics.is_empty() {
      let err_msg = ret
        .diagnostics
        .into_iter()
        .map(|d| d.to_string())
        .collect::<Vec<_>>()
        .join("\n");
      return Err(napi::Error::new(napi::Status::GenericFailure, err_msg));
    }

    let header = format!("// Compiled: {}\n", path.display());
    Ok(format!("{}{}", header, source_text))
  }
}

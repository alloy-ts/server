# Rocket NAPI Bindings Implementation Status

This document tracks the current implementation status of Rocket web framework NAPI bindings in `@alloy-ts/server`.

---

## Implemented Components

| Component                   | Rust File          | NAPI / TS Exports                                                                                | Status      |
| --------------------------- | ------------------ | ------------------------------------------------------------------------------------------------ | ----------- |
| **Rocket Configuration**    | `src/config.rs`    | `Config`, `ConfigInit`, `Limits`, `Shutdown`, `TlsConfig`, `MutualTls`, parameter constants      | ✅ Complete |
| **Route Metadata**          | `src/route.rs`     | `Route` NAPI class (`new`, `ranked`, getters/setters)                                            | ✅ Complete |
| **Route URI Metadata**      | `src/route_uri.rs` | `RouteUri` NAPI class (`new`, `base`, `path`, `query`, `asStr`, `defaultRank`)                   | ✅ Complete |
| **Route Handler Outcomes**  | `src/handler.rs`   | `HandlerOutcome`, `outcomeSuccess`, `outcomeError`, `outcomeForward`                             | ✅ Complete |
| **Alloy.serve HTTP Server** | `src/server.rs`    | `serve`, `Response`, `response`, `responseJson`, `responseRedirect`, `file`, `Server`            | ✅ Complete |
| **App Builder**             | `src/server.rs`    | `App` class (`app.get()`, `app.post()`, `app.launch()`)                                          | ✅ Complete |
| **Codegen Utilities**       | `src/codegen.rs`   | `parseRoute`, `parseRoutes`, `toRocketPath`, `generateClient`, `generateTypes`, `generateModule` | ✅ Complete |

---

## Architecture & Binding Directives

- **Zero TS Wrapper Indirection:** All NAPI classes, functions, and interfaces are generated directly from Rust `#[napi]` annotations into `dist/index.js` and `dist/index.d.ts`.
- **Panic Safety:** Route and RouteUri constructors wrap Rust calls in `std::panic::catch_unwind` to return `napi::Result` errors on invalid URI syntax instead of panicking FFI.

---

## Verification & Test Status

- **Rust Unit Tests (`cargo test`)**: 17 tests passed.
- **TypeScript Unit Tests (`npm test`)**: 22 Vitest tests + 1 CJS test passed.
- **Code Check & Linting (`npm run check`, `npm run fmt`)**: 0 errors, 0 warnings.

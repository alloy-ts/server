# `@alloy-ts/server` Implementation Status & Assessment

This document outlines the current implementation status of Rocket NAPI bindings for Node.js/TypeScript, answers key evaluation questions, and maps out the future roadmap.

---

## Executive Summary: "Do we have Rocket working in TypeScript properly?"

**Yes!** Rocket is running smoothly and reliably inside Node.js/TypeScript through native NAPI bindings (`alloy-server-native`):
- **Runtime Model**: Rocket launches on a dedicated multi-threaded Tokio runtime in a background OS worker thread. Node's main V8 event loop remains non-blocking.
- **Request/Response Data Bridge**: Incoming HTTP requests convert to native JS request contexts (`method`, `path`, `query`, `headers`, `params`, `.json()`, `.text()`).
- **Async Threadsafe Function Dispatch**: JS route handlers resolve async promises back to Rocket worker threads safely.
- **Independent Life Cycles**: Server instances created via `Alloy.serve`, `Bun.serve`, or `App.launch()` carry individual graceful shutdown handles (`server.stop()`).

---

## Feature Implementation Matrix

| Component / Feature | Rocket Rust Source | NAPI Binding Module | Status | Notes & Verification |
| :--- | :--- | :--- | :--- | :--- |
| **Server Bootstrapper (`Alloy.serve` / `Bun.serve` / `serve`)** | `rocket::Rocket` & `rocket::launch` | `src/server.rs` | ✅ **Complete** | Options object (`routes`, `fetch`, `port`, `hostname`, `unix`), dynamic routes, per-method maps, wildcards, redirects. |
| **Application Builder (`App`)** | `rocket::build()`, `#[get]`, `routes!`, `#[launch]` | `src/server.rs` | ✅ **Complete** | Fluent API (`app.get`, `app.post`, `app.put`, `app.delete`, `app.patch`, `app.head`, `app.options`, `app.mount`, `app.launch`). |
| **Response Helpers** | `rocket::response::Response` & `Responder` | `src/server.rs` | ✅ **Complete** | `response()`, `responseJson()`, `responseRedirect()`. |
| **Static File Serving** | `rocket::fs::FileServer` / `NamedFile` | `src/server.rs` | ✅ **Complete** | `file()`, `Alloy.file()`, `Bun.file()` lazy disk file serving. |
| **Route Metadata & Ranking Matrix (`Route`)** | `rocket::Route` | `src/route.rs` | ✅ **Complete** | Full Rocket default rank matrix computation (`-12` to `-1`), `Route.ranked()`, and `map_base()` support. |
| **Custom Handlers & Outcomes (`Handler` & `Outcome`)** | `rocket::route::Handler` & `Outcome` | `src/handler.rs` | ✅ **Complete** | `Handler` class, `outcomeFrom()`, `outcomeError()`, `outcomeForward()`. |
| **Rocket Configuration (`Config`)** | `rocket::config::Config` | `src/config.rs` | ✅ **Complete** | `Config`, `ConfigInit`, profiles, workers, max blocking threads, keep-alive, limits, shutdown. |
| **TypeScript Codegen (`codegen`)** | `rocket_codegen` | `src/codegen.rs` | ✅ **Complete** | `parseRoute`, `parseRoutes`, `toRocketPath`, `generateClient`, `generateModule`, `generateTypes`. |
| **Example Scripts** | - | `examples/server.ts`, `examples/app-routes.ts` | ✅ **Complete** | Verified execution with `node --import @oxc-node/core/register`. |

---

## Test Coverage & Verification

- **Vitest Unit Tests**: `src/server.test.ts`, `src/config.test.ts`, `src/route.test.ts`, `src/handler.test.ts`, `src/codegen.test.ts`, `src/main.test.ts` (19 passing tests).
- **CommonJS Smoke Tests**: `test.cjs` (verifying native addon loading under Node CJS require).
- **Cargo Rust Tests**: `cargo test` (8 passing unit tests).
- **TypeScript Runner Verification**: `node --import @oxc-node/core/register examples/server.ts` & `examples/app-routes.ts`.

---

## What's Missing / Next Steps Roadmap

While core HTTP serving, routing, configuration, route metadata ranking, custom outcomes, and client code generation are fully implemented and verified, the following advanced Rocket subsystems can be expanded in future iterations:

1. **Fairings NAPI Bindings (`rocket::fairing::Fairing`)**:
   - Expose lifecycle hooks (`on_ignite`, `on_liftoff`, `on_request`, `on_response`, `on_shutdown`) so JS plugins can inspect or mutate requests/responses globally.
2. **Error Catcher NAPI Bindings (`rocket::catcher::Catcher`)**:
   - Expose custom status error catchers (`#[catch(404)]`, `catchers!`) as NAPI classes.
3. **Streaming & Server-Sent Events (SSE)**:
   - Support `ByteStream` / `TextStream` / SSE streaming responses directly over NAPI.
4. **Form & Data Guard Bindings (`rocket::form::FromForm`)**:
   - Expose multipart/form-data parsing guards for complex body validation.
5. **Managed State Bindings (`rocket::State<T>`)**:
   - Allow JS objects to be registered in Rocket's managed state registry and retrieved across handlers.

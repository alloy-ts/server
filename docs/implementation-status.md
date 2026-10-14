# Implementation Status: Rocket NAPI Bindings for TypeScript

This document outlines the current state of `@alloy-ts/server` (NAPI bindings for Rocket v0.5), features currently implemented, verified test coverage, identified gaps, and the recommended roadmap for completing full Rocket support.

---

## Current Status Summary

`@alloy-ts/server` successfully bridges the Rocket web framework to Node.js and TypeScript via `@napi-rs/cli`.

The core HTTP engine runs Rocket v0.5 on a dedicated Tokio multi-threaded background runtime. Request processing, parameter extraction, route routing, static file serving, and graceful shutdown are fully operational through a JS-friendly API compatible with Bun.serve and Rocket's native builder paradigms.

---

## Supported Features

### 1. HTTP Server Runtime (`src/server.rs` & `src/main.ts`)

- [x] **`Alloy.serve()`**: Bun.serve-compatible server entrypoint supporting route tables, per-method route maps, wildcard route matching (`/api/*`), static files (`file()`), redirects (`Response.redirect`), and fallback `fetch` handlers.
- [x] **`App` Builder Class**: Object-oriented builder mirroring Rocket's `rocket::build().mount(...).launch()` pipeline.
- [x] **`Server` Lifecycle Handle**: Asynchronous launching on background thread and graceful shutdown via `server.stop()`.
- [x] **Route Parameter Extraction**: Extraction of named path parameters (`:id`, `<id>`, `<id..>`) into `req.params`.

### 2. Rocket Configuration (`src/config.rs`)

- [x] **`Config` NAPI Class**: Direct NAPI mirror of `rocket::config::Config`.
- [x] **Default Profiles**: `Config.debug_default()`, `Config.release_default()`, `Config.default_config()`.
- [x] **Config Parameters & Overrides**: Support for `address`, `port`, `workers`, `max_blocking`, `keep_alive`, `ident`, `ip_header`, `limits`, `temp_dir`, `shutdown`, `log_level`, and `cli_colors`.
- [x] **`Config.try_from()` & `Config.from()`**: Partial override initialization.

### 3. Route Decorators & Codegen (`src/main.ts`)

- [x] **Route Method Decorators**: `@Get`, `@Post`, `@Put`, `@Delete`, `@Patch`, `@Head`, `@Options`, `@Route`, and `@Catch`.
- [x] **Type-Safe URI Generation**: `uri()` and `buildRouteUri()` parameter replacement and URL encoding.
- [x] **Route Pattern Parsing**: `parseRoutePattern()` syntax analysis for Rocket path and query parameters.
- [x] **TypeScript Codegen**: `generateRoutesTypeDefinitions()` generating TypeScript `ApiRoutes` interface definitions from route lists.

---

## Identified Gaps & Missing Features

To reach complete 1:1 parity with all of Rocket's advanced Rust features across the NAPI boundary, the following areas remain to be expanded:

1. **State Management (`rocket::State`)**:
   - _Current status_: Handlers store state in JS closures.
   - _Gap_: Rocket's native state container (`Rocket::manage`) is not yet exposed via NAPI to pass managed state directly from Rust to JS callbacks.

2. **Fairings / Middleware Hooks (`rocket::fairing::Fairing`)**:
   - _Current status_: Internal shutdown fairing is attached.
   - _Gap_: JS-side fairing registration (`on_request`, `on_response`, `on_shutdown`) is not yet wired to Rocket's fairing pipeline.

3. **Streaming & WebSockets**:
   - _Current status_: Requests/responses buffer up to 1 MiB.
   - _Gap_: Chunked HTTP response streaming and WebSocket upgrade hooks (`rocket::response::stream`) are not yet bridged.

4. **Multi-Part & Form Data Guards (`rocket::form::Form`)**:
   - _Current status_: Bodies are received as raw bytes (`Uint8Array`).
   - _Gap_: High-level multipart/form-data parsing helpers on `req` are not yet included in the JS layer.

5. **Custom Catchers (`#[catch(status)]`)**:
   - _Current status_: Standard 404/500 Rocket catchers handle errors; `@Catch` decorator is a stub.
   - _Gap_: Custom status-code catchers (e.g. 401, 403, 404) are not yet dispatched through NAPI to user-defined JS catcher handlers.

---

## Roadmap for Finishing NAPI Bindings

### Next Step Plan

1. **Phase 1: Custom Catcher Dispatch**
   - Bind Rocket's `Catcher` API through NAPI so custom status code handlers (400, 401, 403, 404, 500) registered via `app.catch(status, handler)` or `@Catch(status)` are dispatched.

2. **Phase 2: Managed State (`Alloy.manage`)**
   - Implement `Alloy.manage(key, value)` allowing JS handlers to access managed state from `req.state(key)`.

3. **Phase 3: Fairing Hook Registration**
   - Expose `Alloy.attach({ onRequest, onResponse, onShutdown })` to hook into Rocket's fairing lifecycle.

4. **Phase 4: Streaming Responses**
   - Support `ReadableStream` responses in JS for long-polling and Server-Sent Events (SSE).

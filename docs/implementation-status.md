# Rocket TypeScript / Node.js Implementation Status & Roadmap

This document outlines the current state of NAPI bindings for the Rocket web framework (`@alloy-ts/server`), what features are fully supported, what's missing, and the roadmap/plan for completing Rocket NAPI integration.

---

## 1. Supported Features

### Core HTTP Server Engine

- **Rocket Runtime Integration:** Driven by Rocket v0.5 and Tokio multi-threaded async runtime via NAPI bindings (`src/server.rs`).
- **`Alloy.serve` Facade API:** Bun/Alloy-compatible server options (`routes` table with static responses, dynamic function handlers, method maps, catch-all fallback `fetch`, redirects, and file responses).
- **`App` Router Interface:** `new App().get(...).post(...).launch()` 1:1 builder pattern corresponding to `rocket::build()`.

### TypeScript Codegen & Decorators

- **Route Decorators:** `@get`, `@post`, `@put`, `@del` method decorators for class controllers.
- **`uri()` Helper:** Parameter interpolation helper matching Rocket's `uri!` macro semantics (`:param` and `<param>`).
- **`Response` & `RequestInput`:** Web-friendly `Response` class (`Response.json`, `Response.redirect`) and `RequestInput` payload passing method, path, query, headers, raw body, and path params (`req.params`).

### Rocket Configuration & Route Metadata

- **NAPI `Config` Class:** Exposure for Rocket's `Config` struct (`port`, `address`, `workers`, `max_blocking`, `ident`, `ip_header`, `limits`, `temp_dir`, `keep_alive`, `shutdown`, `log_level`, `cli_colors`).
- **NAPI `Route` Class:** Exposure for Rocket's `Route` metadata (`method`, `uri`, `rank`, `name`, `format`).
- **Defaults & Figment Integration:** `Config.debug_default()`, `Config.release_default()`, `Config.figment()`, and `Config.try_from(init)`.

---

## 2. Missing Features / Next Steps Plan

To achieve complete feature parity with Rocket's full Rust ecosystem, the following NAPI capabilities are planned for upcoming phases:

### Phase 1: Request Guards & Managed State (`State`)

- **Status:** Planned
- **Goal:** Allow JS route handlers to access global managed state attached to Rocket (`rocket.manage(state)`).
- **Design:** Expose `Rocket.manage(key, value)` and pass `state` map in `RequestInput`.

### Phase 2: Custom Catchers & Error Handlers

- **Status:** Partially supported in `Rocket` handle class; planned for `Alloy.serve` options.
- **Goal:** Allow custom error handlers for HTTP status codes (`#[catch(404)]`, `#[catch(500)]`, default catchers).
- **Design:** Support `catchers: { 404: (req) => ..., default: (req) => ... }` in `ServeOptions`.

### Phase 3: Fairings / Middleware System

- **Status:** Planned
- **Goal:** Expose Rocket fairings (`AdHoc.on_request`, `AdHoc.on_response`, `AdHoc.on_shutdown`) to TypeScript.
- **Design:** Expose `app.attach({ onRequest, onResponse, onShutdown })`.

### Phase 4: WebSockets & Server-Sent Events (SSE) Streaming

- **Status:** Planned
- **Goal:** Streaming responses (`AsyncRead` / `Stream` bodies) and WebSocket upgrading via Rocket's upgrade handlers.
- **Design:** Allow `Response` bodies to take `ReadableStream` or AsyncIterators, and add `server.upgrade(req, wsHandler)`.

### Phase 5: TLS & Mutual TLS (mTLS) Bindings

- **Status:** Gated behind crate feature flags in `Cargo.toml`.
- **Goal:** Expose TLS certificate path configuration (`certs`, `key`, `mutual.ca_certs`) to Node.js when Rocket is built with `features = ["tls", "mtls"]`.

---

## Summary Matrix

| Rocket Feature           | TypeScript Binding Status | Details                                    |
| ------------------------ | ------------------------- | ------------------------------------------ |
| HTTP Server Runtime      | ✅ Supported              | Rocket v0.5 + Tokio multi-threaded runtime |
| Bun / Alloy.serve API    | ✅ Supported              | `Alloy.serve({ routes, fetch, port })`     |
| Route Decorators         | ✅ Supported              | `@get`, `@post`, `@put`, `@del`            |
| URI Helper               | ✅ Supported              | `uri("/users/:id", { id: 1 })`             |
| Static Files             | ✅ Supported              | `Alloy.file(path)`                         |
| Configuration (`Config`) | ✅ Supported              | Full `Config` class & `ConfigInit`         |
| Route Metadata (`Route`) | ✅ Supported              | `Route` class & default rank calculation   |
| Managed State            | ⏳ Planned                | Phase 1                                    |
| Custom Catchers          | ⏳ Planned                | Phase 2                                    |
| Fairings / Middleware    | ⏳ Planned                | Phase 3                                    |
| WebSockets / Streaming   | ⏳ Planned                | Phase 4                                    |
| TLS / mTLS               | ⏳ Feature-Gated          | Phase 5                                    |

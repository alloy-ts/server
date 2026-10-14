# Rocket NAPI Bindings — Implementation Status & Roadmap

This document summarizes the implementation status of Rocket NAPI bindings for TypeScript (Node.js), reviewing implemented features, architectural details, missing capabilities, and the recommended roadmap for future steps.

---

## 1. Implemented Features Review

### A. Bun/Alloy-Compatible HTTP Server (`Alloy.serve` / `serve`)

- **API Surface**: `serve({ port, hostname, unix, routes, fetch })`
- **Route Translators**: Translates Bun/Alloy patterns (`/users/:id`, `/api/*`) into Rocket route patterns (`/users/<id>`, `/api/<_..>`).
- **Route Value Support**:
  - Direct `Response` objects
  - Static file lazy serving (`file("./path")`)
  - Per-method maps (`{ GET: ..., POST: ... }`)
  - Dynamic JavaScript functions `(req) => Response | Promise<Response>`
- **Response Builders**: `Response.new()`, `Response.json()`, `Response.redirect()`.
- **Server Lifecycle**: `server.port`, `server.url`, and graceful shutdown `server.stop()`.

### B. Rocket Lifecycle API

- **Class `Rocket`**: Instantiates a custom `Rocket<Build>` server.
- **`mount(base, method, path, handler)`**: Mounts individual handlers directly onto Rocket's router.
- **`register(base, code, handler)`**: Registers HTTP status catchers (404, 500, or default).
- **`launch()` & `shutdown()`**: Ignites, launches on a dedicated Tokio multi-thread runtime, and executes graceful shutdown.

### C. Server Configuration (`Config` & `ConfigInit`)

- **Class `Config`**: Faithfully mirrors `rocket::config::Config`.
- **Static Factories**: `debugDefault()`, `releaseDefault()`, `defaultConfig()`, `figment()`.
- **Properties**: `profile`, `address`, `port`, `workers`, `maxBlocking`, `ident`, `ipHeader`, `limits`, `tempDir`, `keepAlive`, `shutdown`, `logLevel`, `cliColors`.
- **Partial Overrides**: `Config.from(init)` and `Config.tryFrom(init)` via `ConfigInit`.

### D. Rocket Code Generation Engine (`codegen`)

- **Route Generation (`generateRoute`, `generateRoutes`)**: Parses route patterns, identifies dynamic parameters (`<param>`, `<param..>`, `:param`, `*`), and extracts parameter type specifications.
- **Catcher Generation (`generateCatcher`)**: Generates error catcher specifications.
- **Type-Safe URI Generation (`generateUri`, `uri`)**: Dynamic URL-safe URI builder matching Rocket's `uri!` macro semantics (path parameter substitution, query parameter assembly, prefix/suffix handling, and URL encoding).
- **TypeScript Declaration Generator (`generateTsDeclarations`)**: Emits TypeScript interface definitions for typed routes.
- **Macro Decorators & Helpers**: `get`, `post`, `put`, `del`, `route`, `routes`, `catchers`, `uri`.
- **Debug Logging**: Supports `ROCKET_CODEGEN_DEBUG=1` compile-time and runtime console diagnostics.

---

## 2. What Is Missing?

To achieve 100% feature parity with Rust Rocket and Bun full-stack server APIs, the following capabilities are currently missing or planned for future steps:

1. **Managed Application State (`State<T>`)**:
   - Ability to call `rocket.manage(state)` from JS and receive injected `req.state` or `State` parameters in handlers.
2. **Fairings Engine**:
   - Exposing Rocket's `Fairing` trait (`on_ignite`, `on_liftoff`, `on_request`, `on_response`, `on_shutdown`) as NAPI callbacks.
3. **Streaming & Server-Sent Events (SSE)**:
   - Chunked streaming responses, `ReadableStream` bridging, and SSE (`event-stream`) timeout overrides (`server.timeout(req, seconds)`).
4. **Form Parsing & Validation (`FromForm`, `FromFormField`)**:
   - Multipart and URL-encoded form parser bindings with schema validation.
5. **WebSocket Support (`rocket_ws`)**:
   - HTTP/1.1 WebSocket upgrade handlers and channel pub/sub.
6. **TLS & Mutual TLS (`tls`, `mtls`)**:
   - Gated TLS certificate configuration and `Certificate` request guard bindings.

---

## 3. Planned Next Steps for Finishing NAPI Bindings for Rocket

1. **Step 1: Managed State & Dependency Injection**:
   - Implement `Rocket.manage(state: Object)` in Rust/NAPI.
   - Attach state object to `RequestPayload` so handlers can access global shared state.

2. **Step 2: Fairings API for JS/TS**:
   - Expose `Rocket.attach(fairing)` allowing JavaScript functions to hook into request/response lifecycles.

3. **Step 3: Response Streaming & Async Generators**:
   - Bridge Node.js `Readable` streams and async generators into Rocket's `response::stream`.

4. **Step 4: Form & Body Validation**:
   - Add NAPI form parsing utilities for multipart upload handling and JSON schema validation.

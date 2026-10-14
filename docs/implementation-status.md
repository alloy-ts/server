# Rocket TypeScript Implementation Status (`@alloy-ts/server`)

This document details the architectural design, implementation status, and feature matrix of the Rocket web framework NAPI bindings for TypeScript and Node.js.

---

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                 TypeScript / Node.js Layer                  │
│       Alloy.serve() / App / Response / Config / uri()       │
└──────────────────────────────┬──────────────────────────────┘
                               │ NAPI Bridge
┌──────────────────────────────▼──────────────────────────────┐
│                     Rust NAPI Layer                         │
│           (napi-rs 3.14 + rocket 0.5.1 cdylib)             │
│   src/server.rs  •  src/rocket_app.rs  •  src/config.rs      │
└──────────────────────────────┬──────────────────────────────┘
                               │ Threadsafe Callbacks / MPSC
┌──────────────────────────────▼──────────────────────────────┐
│                Rocket HTTP Server Engine                    │
│            Multithreaded Tokio Runtime Worker               │
└─────────────────────────────────────────────────────────────┘
```

---

## Feature Matrix

| Category             | Feature                      |    Status    | Notes                                                       |
| :------------------- | :--------------------------- | :----------: | :---------------------------------------------------------- |
| **Server Engine**    | Rocket 0.5.1 HTTP Core       | ✅ Supported | Runs on dedicated Tokio multithreaded worker                |
|                      | Synchronous Port Resolution  | ✅ Supported | MPSC channel syncs bound port (supports `port: 0`)          |
|                      | Instance-bound Shutdown      | ✅ Supported | `Server.stop()` triggers graceful Rocket shutdown           |
|                      | Environment Port Resolution  | ✅ Supported | Resolves `$BUN_PORT`, `$PORT`, `$NODE_PORT`, or `3000`      |
| **Routing**          | Static Routes                | ✅ Supported | Handled via `ResponsePayload`                               |
|                      | Dynamic Route Parameters     | ✅ Supported | Extracted from `:param` and `<param>` into `req.params`     |
|                      | Wildcard Routes              | ✅ Supported | Translates `/prefix/*` into Rocket `<_..>` catch-all        |
|                      | Per-Method Route Maps        | ✅ Supported | Supports `{ GET, POST, PUT, DELETE, PATCH }` tables         |
|                      | Redirect Responses           | ✅ Supported | `Response.redirect(url, status)`                            |
|                      | Static File Serving          | ✅ Supported | `Alloy.file(path)` serves static files                      |
|                      | Fallback Handler             | ✅ Supported | `fetch(req)` fallback for unmatched routes                  |
| **Configuration**    | Config Profiles              | ✅ Supported | `debugDefault()`, `releaseDefault()`, `defaultConfig()`     |
|                      | Property Accessors           | ✅ Supported | `address`, `port`, `workers`, `maxBlocking`, `limits`, etc. |
|                      | Figment Provider Integration | ✅ Supported | `Config.figment()` extracts configuration                   |
|                      | Override Instantiation       | ✅ Supported | `Config.tryFrom(init)` and `Config.from(init)`              |
| **Codegen & Macros** | URL Construction             | ✅ Supported | `uri(pattern, params)` with automatic URL encoding          |
|                      | Route Pattern Parser         | ✅ Supported | `parseRoutePattern` extracts path and query metadata        |
|                      | TypeScript Interface Gen     | ✅ Supported | `generateTsRouteTypes` emits `ApiRoutes` interfaces         |
|                      | Route Decorators             | ✅ Supported | `@get`, `@post`, `@put`, `@del`, `@route`                   |
|                      | Metadata Inspection          | ✅ Supported | `getRegisteredRoutesMeta()` tracks registered routes        |

---

## Module Breakdown

### 1. `src/server.rs`

- Implements `serve()` for `Alloy.serve` / `Bun.serve` compatibility.
- Maps JS handlers, static responses, files, and per-method route maps into Rocket `Route` instances.
- Manages multithreaded Tokio worker threads with `std::sync::mpsc::channel` for port binding synchronization.
- Stores per-instance `Shutdown` handles for `server.stop()`.

### 2. `src/config.rs`

- Provides `#[napi]` wrapper around Rocket's `Config` struct.
- Exposes limits (`Limits`), shutdown flags (`Shutdown`), and parameter name constants.

### 3. `src/codegen.rs` & `src/codegen.ts`

- Provides `parse_route_pattern`, `build_route_uri`, and `generate_ts_route_types`.
- Exposes TypeScript helpers `uri`, `route`, `get`, `post`, `put`, `del`, and `generateRoutesTypeDefinitions`.

### 4. `src/main.ts`

- Exposes `Alloy` namespace (`Alloy.serve`, `Alloy.file`, `Alloy.App`, `Alloy.response`).
- Exposes `App` class mirroring Rocket's `rocket::build()` builder pattern.

---

## Test Verification

- **Vitest Suite (`vp test`)**: 22 unit tests across `src/codegen.test.ts`, `src/config.test.ts`, `src/server.test.ts`, and `src/main.test.ts`.
- **Node CJS Integration (`test.cjs`)**: Verified CommonJS module loading of `dist/index.js`.
- **TypeScript / Format Verification (`vp check`)**: Passes clean without lint or type errors.
- **OXC-Node Integration (`@oxc-node/core`)**: Verified running TypeScript examples directly with `node --import @oxc-node/core/register`.

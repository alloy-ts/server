# Implementation Status: Rocket NAPI Bindings & TypeScript Codegen

This document describes the implementation status, architecture, and feature matrix of `@alloy-ts/server` (native Rust package `alloy-server-native`).

---

## 1. Architecture

The codebase bridges Rust's **Rocket v0.5** web framework to Node.js / TypeScript using `@napi-rs/cli` and `napi-derive`:

```
┌─────────────────────────────────────────────────────────────┐
│                   TypeScript / Node.js                      │
│   `Alloy.serve`  /  `App`  /  `Rocket`  /  `Config`  /  SDK   │
└──────────────────────────────┬──────────────────────────────┘
                               │ NAPI (napi-rs)
┌──────────────────────────────▼──────────────────────────────┐
│                    Rust Bridge Modules                      │
│   codegen.rs  │  config.rs  │  rocket.rs  │  server.rs       │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│                  Rocket v0.5 Runtime                        │
│           (Tokio multi-threaded background executor)        │
└─────────────────────────────────────────────────────────────┘
```

- **Thread safety**: JS handlers are bridged into Rocket's multi-threaded Tokio runtime via `napi::threadsafe_function::ThreadsafeFunction`.
- **Non-blocking event loop**: `serve()` and `App.launch()` boot Rocket on a background Tokio multi-thread runtime so Node's main event loop remains non-blocking.

---

## 2. Feature Matrix & Implementation Status

| Category             | Feature / API                          |   Status    | Notes                                                                                                 |
| :------------------- | :------------------------------------- | :---------: | :---------------------------------------------------------------------------------------------------- |
| **Server API**       | `Alloy.serve()` / `serve()`            | ✅ Complete | Bun/Alloy-compatible declarative routing                                                              |
|                      | Static & dynamic routes (`/users/:id`) | ✅ Complete | Dynamic path parameters extracted into `req.params`                                                   |
|                      | Per-HTTP method handlers               | ✅ Complete | `{ GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS }`                                                    |
|                      | Wildcard routes (`/api/*`)             | ✅ Complete | Express/Alloy wildcard pattern translation                                                            |
|                      | File serving (`Alloy.file()`)          | ✅ Complete | Lazily served static files                                                                            |
|                      | Redirects (`Response.redirect()`)      | ✅ Complete | 302/3xx HTTP redirects                                                                                |
|                      | Fallback `fetch` handler               | ✅ Complete | Unmatched request handler                                                                             |
| **App Builder**      | `App` class                            | ✅ Complete | Express/Rocket-style builder (`app.get()`, `app.post()`, `app.mount()`, `app.launch()`, `app.stop()`) |
| **Rocket Lifecycle** | `Rocket` class                         | ✅ Complete | Direct NAPI control over ignition, mounting, catchers, launch, and shutdown                           |
|                      | Route mounting                         | ✅ Complete | `rocket.mount(base, method, path, handler)`                                                           |
|                      | Catcher registration                   | ✅ Complete | `rocket.register(base, status, handler)`                                                              |
|                      | Graceful shutdown                      | ✅ Complete | Programmatic shutdown notification                                                                    |
| **Configuration**    | `Config` class                         | ✅ Complete | Full mirror of `rocket::config::Config`                                                               |
|                      | Profile defaults                       | ✅ Complete | `Config.debugDefault()`, `Config.releaseDefault()`, `Config.defaultConfig()`                          |
|                      | Configuration overrides                | ✅ Complete | `Config.tryFrom(init)`                                                                                |
|                      | Parameter inspection                   | ✅ Complete | `Config.parameters()`                                                                                 |
|                      | Limits & Shutdown config               | ✅ Complete | `Limits`, `Shutdown` structures                                                                       |
| **Codegen**          | `parseRocketRoutePattern`              | ✅ Complete | Extracts path & query parameters                                                                      |
|                      | `translatePatternToAlloy`              | ✅ Complete | Converts `/users/<id>/<rest..>` to `/users/:id/*`                                                     |
|                      | `toRocketPath`                         | ✅ Complete | Converts `/users/:id/*` to `/users/<id>/<_..>`                                                        |
|                      | `generateTypescriptClient`             | ✅ Complete | Emits strongly-typed fetch client SDK                                                                 |
|                      | `generateRouteHandler`                 | ✅ Complete | Emits handler boilerplate                                                                             |
|                      | `generateTypescriptInterface`          | ✅ Complete | Converts field definitions into TS interfaces                                                         |
|                      | `generateTypes` & `generateModule`     | ✅ Complete | Emits `.d.ts` route map and complete TS module                                                        |

---

## 3. Testing & Verification

The implementation is verified across three test layers:

1. **Rust Cargo Tests (`cargo test`)**:
   - 10 unit tests covering pattern parsing, path translation, reserved word sanitization, route name deduplication, interface generation, and TypeScript code emission.
2. **TypeScript Vite-Plus Tests (`npm test` / `vp test`)**:
   - 13 unit tests in `src/main.test.ts` and `src/config.test.ts` verifying `main()`, `httpMethods()`, `parseRocketRoutePattern()`, `translatePatternToAlloy()`, `parseRoute()`, `parseRoutes()`, `generateTypes()`, `generateModule()`, `generateTypescriptClient()`, `App` route registration & server launch, and `Config` default/override mutation.
3. **CommonJS NAPI Integration Tests (`test.cjs`)**:
   - Runtime integration test suite verifying CommonJS loading of compiled native bindings in `dist/`.

---

## 4. Examples Provided

The `examples/` directory contains full runnable TypeScript examples:

- `examples/server.ts`: Bun/Alloy-compatible server setup demo.
- `examples/app-routes.ts`: Quickstart Rocket `App` builder demo.
- `examples/codegen.ts`: Route pattern parsing, path translation, client SDK, and interface generation demo.
- `examples/config.ts`: Rocket server configuration (`Config`) demo.
- `examples/rocket-lifecycle.ts`: Programmatic Rocket server lifecycle demo.

All examples can be executed directly using:

```bash
node --import @oxc-node/core/register examples/<script>.ts
```

---

## 5. What's Missing & Next Steps Roadmap

While Rocket's core HTTP server, route mounting, configuration, and codegen are fully working in TypeScript, future extensions could further expand Rocket's advanced features:

1. **Request Fairings & Middleware Bridge**:
   - Expose Rocket's `Fairing` lifecycle hooks (`on_request`, `on_response`, `on_shutdown`) so JS/TS middleware functions can inspect or attach headers across all incoming requests and responses.
2. **Streaming Requests & Form/Multipart Guards**:
   - Add high-level JS helpers for handling streaming request bodies (`ReadableStream`) or parsing `multipart/form-data` uploads directly into structured objects.
3. **WebSockets / HTTP Upgrade Bridge**:
   - Bridge Rocket 0.5's connection upgrade / WebSocket handlers into JS callbacks for real-time bidirectional communication.

# Implementation Status: Rocket NAPI Bindings & TypeScript Support

This document details the current implementation status of Node.js / TypeScript NAPI bindings for the [Rocket web framework](https://rocket.rs/) in `@alloy-ts/server`.

---

## Executive Summary

| Feature Area                              |   Status    | Implementation Details                                                                                                                                                                                |
| :---------------------------------------- | :---------: | :---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Rocket Core Server (`Rocket`)**         | ✅ Complete | Full NAPI lifecycle (`new`, `mount`, `register`, `launch`, `shutdown`) wrapping `Rocket<Build>`.                                                                                                      |
| **HTTP Server (`Alloy.serve` / `serve`)** | ✅ Complete | Bun/Alloy-compatible declarative route map server with fallback `fetch`.                                                                                                                              |
| **Port Resolution**                       | ✅ Complete | Explicit `port` option + fallback order: `BUN_PORT` → `PORT` → `NODE_PORT` → `3000`.                                                                                                                  |
| **Rocket Configuration (`Config`)**       | ✅ Complete | Full mirror of Rocket's Figment-backed `Config` (`debugDefault`, `releaseDefault`, `tryFrom`, `from`, parameter constants).                                                                           |
| **Error Catchers**                        | ✅ Complete | Registration of status-specific (404, 500) and default catchers (`rocket.register()`).                                                                                                                |
| **TypeScript Codegen (`codegen`)**        | ✅ Complete | Runtime pattern parsing (`parseRoute`), syntax translation (`toRocketPath`), type emission (`generateTypes`), client SDK (`generateClient`), and self-contained module generation (`generateModule`). |
| **Cross-Thread Dispatch**                 | ✅ Complete | Non-blocking background Tokio multi-thread runtime bridged via `ThreadsafeFunction`.                                                                                                                  |
| **Testing Coverage**                      | ✅ Complete | Unit tests covering codegen parsing, emission, type bounds, and configuration mutation (`src/codegen.test.ts`, `src/config.test.ts`).                                                                 |

---

## Detailed Feature Breakdown

### 1. HTTP Server API (`serve` / `Alloy.serve`)

- ✅ **Static String & JSON Responses**: Handlers returning `response("text")` or `responseJson({ ... })`.
- ✅ **Dynamic Path Parameters**: Dynamic segment extraction (`/users/:id` → `req.params.id`).
- ✅ **Per-Method Routing**: Mapping HTTP methods (`GET`, `POST`, `PUT`, `DELETE`, `PATCH`, `HEAD`, `OPTIONS`) per route key.
- ✅ **Wildcard Trailing Routes**: Pattern `/api/*` translated to Rocket's `<_..>` catch-all segment.
- ✅ **Redirects**: `responseRedirect(url, status)` generating `Location` headers.
- ✅ **Static File Serving**: Lazy static file serving marker (`file(path)`).
- ✅ **Fallback Handler**: `fetch(req)` fallback callback for unmatched routes.

### 2. Rocket Lifecycle & Programmatic Control (`Rocket` Class)

- ✅ **Instance Initialization**: `new Rocket(config)` converting NAPI scalar config into Rocket `Config`.
- ✅ **Route Mounting**: `rocket.mount(base, method, path, handler)`.
- ✅ **Catcher Registration**: `rocket.register(base, code, handler)` for status codes in `[400, 600)` or default.
- ✅ **Ignition & Launch**: `rocket.launch()` executing on dedicated Tokio runtime.
- ✅ **Graceful Shutdown**: `rocket.shutdown()` notifying Rocket's cancellation tripwire.

### 3. Rocket Configuration (`Config` Class)

- ✅ **Profile Presets**: `Config.debugDefault()`, `Config.releaseDefault()`, `Config.defaultConfig()`.
- ✅ **Partial Overrides**: `Config.tryFrom(init)` merging partial overrides onto defaults.
- ✅ **Getters & Setters**: Read/write access for `address`, `port`, `workers`, `maxBlocking`, `keepAlive`, `ident`, `ipHeader`, `limits`, `tempDir`, `logLevel`, `shutdown`, `cliColors`.
- ✅ **Validation**: Eager IP address and log-level syntax validation on setters.

### 4. Code Generation (`codegen` Module)

- ✅ **Route Parsing**: `parseRoute` / `parseRoutes` converting route inputs into structured `RouteDef`.
- ✅ **Pattern Translation**: `toRocketPath` translating Bun/Alloy patterns (`:id`, `*`) to Rocket syntax (`<id>`, `<_..>`).
- ✅ **Reserved Keyword Escaping**: Automatic renaming of reserved TS identifiers (`type` → `type_`).
- ✅ **Identifier Sanitization**: Invalid identifier characters converted to safe parameter names.
- ✅ **Name Deduplication**: Automatic disambiguation of duplicate handler names (`getUser`, `getUser_2`).
- ✅ **Type Emission**: `generateTypes` emitting `.d.ts`-style `RouteTypes`, `ParamsOf`, `QueryOf`, `BodyOf`, `ResponseOf`.
- ✅ **Client SDK Generation**: `generateClient` emitting a typed class with `uri()` helper and `fetch` methods.
- ✅ **Module Emission**: `generateModule` emitting unified TypeScript file.

---

## Known Limitations & Future Work

1. **HTTP/2 & WebSocket Upgrade Stream**: Connection upgrade streams (WebSockets) require explicit Rocket `IoHandler` bridging.
2. **Streaming Request/Response Bodies**: Request bodies are capped at 1 MiB in the default wrapper; chunked streaming bindings can be added for ultra-large payloads.
3. **TLS Feature Gating**: TLS and mTLS configuration require compiling Rust binary with `--features tls,secrets`.

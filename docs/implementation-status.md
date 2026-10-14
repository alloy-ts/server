# Implementation Status: Rocket NAPI Bindings & TypeScript Support

This document details the current implementation status of Node.js / TypeScript NAPI bindings for the [Rocket web framework](https://rocket.rs/) in `@alloy-ts/server`.

---

## Executive Summary

| Feature Area                              |   Status    | Implementation Details                                                                                                                                                                                |
| :---------------------------------------- | :---------: | :---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Rocket Core Server (`Rocket`)**         | ✅ Complete | Full NAPI lifecycle (`new`, `mount`, `register`, `launch`, `shutdown`) wrapping `Rocket<Build>`. Generated directly from NAPI macros in Rust (`src/main.rs`).                                         |
| **HTTP Server (`Alloy.serve` / `serve`)** | ✅ Complete | Bun/Alloy-compatible declarative route map server with fallback `fetch`.                                                                                                                              |
| **Catcher Class (`Catcher`)**             | ✅ Complete | NAPI `Catcher` wrapper supporting status error catchers, default catchers, and `mapBase`.                                                                                                             |
| **Route Class (`Route`)**                 | ✅ Complete | NAPI `Route` wrapper with computed/explicit ranks, method, URI, format, name, and `mapBase`.                                                                                                          |
| **RouteUri Class (`RouteUri`)**           | ✅ Complete | NAPI `RouteUri` wrapper exposing `base`, `path`, `query`, and `asStr`.                                                                                                                                |
| **Handler Outcomes (`HandlerBridge`)**    | ✅ Complete | NAPI bridge for Rocket's `Handler` trait supporting `Success`, `Error`, and `Forward` outcomes.                                                                                                       |
| **Port Resolution**                       | ✅ Complete | Explicit `port` option + fallback order: `BUN_PORT` → `PORT` → `NODE_PORT` → `3000`.                                                                                                                  |
| **Rocket Configuration (`Config`)**       | ✅ Complete | Full mirror of Rocket's Figment-backed `Config` (`debugDefault`, `releaseDefault`, `tryFrom`, `from`, parameter constants).                                                                           |
| **TypeScript Codegen (`codegen`)**        | ✅ Complete | Runtime pattern parsing (`parseRoute`), syntax translation (`toRocketPath`), type emission (`generateTypes`), client SDK (`generateClient`), and self-contained module generation (`generateModule`). |
| **Cross-Thread Dispatch**                 | ✅ Complete | Non-blocking background Tokio multi-thread runtime bridged via `ThreadsafeFunction`.                                                                                                                  |
| **Testing Coverage**                      | ✅ Complete | Unit tests covering codegen parsing, emission, configuration, handler outcomes, catchers, route ranking, and route URIs (`src/*.test.ts`).                                                            |

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

### 3. Catcher, Route & Handler Trait Bindings (`Catcher`, `Route`, `RouteUri` & `HandlerBridge`)

- ✅ **Catcher Class**: `new Catcher(code, handler)`, `catcher.base`, `catcher.code`, `catcher.mapBase(base)`.
- ✅ **Route Class**: `new Route(method, uri, handler)`, `Route.ranked(rank, method, uri, handler)`.
- ✅ **Route Inspection**: Getters for `name`, `method`, `uri`, `rank`, and `format`.
- ✅ **Rebasing**: `route.mapBase(base)` returning new rebased `Route`.
- ✅ **RouteUri Inspection**: Read access to `base`, `path`, `query`, and `asStr`.
- ✅ **Custom Handler Outcomes**: `HandlerBridge` supporting `success(response)`, `error(status)`, and `forward(status)`.

### 4. Rocket Configuration (`Config` Class)

- ✅ **Profile Presets**: `Config.debugDefault()`, `Config.releaseDefault()`, `Config.defaultConfig()`.
- ✅ **Partial Overrides**: `Config.tryFrom(init)` merging partial overrides onto defaults.
- ✅ **Getters & Setters**: Read/write access for `address`, `port`, `workers`, `maxBlocking`, `keepAlive`, `ident`, `ipHeader`, `limits`, `tempDir`, `logLevel`, `shutdown`, `cliColors`.
- ✅ **Validation**: Eager IP address and log-level syntax validation on setters.

### 5. Code Generation (`codegen` Module)

- ✅ **Route Parsing**: `parseRoute` / `parseRoutes` converting route inputs into structured `RouteDef`.
- ✅ **Pattern Translation**: `toRocketPath` translating Bun/Alloy patterns (`:id`, `*`) to Rocket syntax (`<id>`, `<_..>`).
- ✅ **Reserved Keyword Escaping**: Automatic renaming of reserved TS identifiers (`type` → `type_`).
- ✅ **Identifier Sanitization**: Invalid identifier characters converted to safe parameter names.
- ✅ **Name Deduplication**: Automatic disambiguation of duplicate handler names (`getUser`, `getUser_2`).
- ✅ **Type Emission**: `generateTypes` emitting `.d.ts`-style `RouteTypes`, `ParamsOf`, `QueryOf`, `BodyOf`, `ResponseOf`.
- ✅ **Client SDK Generation**: `generateClient` emitting a typed class with `uri()` helper and `fetch` methods.
- ✅ **Module Emission**: `generateModule` emitting unified TypeScript file.

---

## What's Missing / Next Steps for Rocket NAPI Bindings

1. **Managed State (`State<T>`)**: Exposing Rocket's managed state container (`rocket.manage(state)`) to TS handlers.
2. **Request Guards**: Exposing custom request guards bridging headers/cookies/auth logic.
3. **Fairings (`Fairing`)**: Support for attaching lifecycle hook callbacks (`on_request`, `on_response`, `on_shutdown`).
4. **Streaming Payload Support**: Support for streaming chunked request/response bodies beyond memory buffers.

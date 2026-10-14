# Implementation Status: Rocket Web Framework NAPI Bindings

## Overview

This repository provides Node.js / TypeScript NAPI bindings for the Rust [Rocket](https://rocket.rs/) web framework (`@alloy-ts/server`). The codebase exposes Rocket's server runtime, configuration, routing, URI parsing, and codegen facilities directly from Rust using `#[napi]` attributes.

---

## Completed Components

### 1. Server Subsystem (`src/server.rs`)

- **`Alloy.serve` / `Bun.serve`**: High-performance HTTP server launcher backed by Rocket's multi-threaded Tokio runtime background thread.
- **Route Table Parsing**: Supports static responses, dynamic route handlers (`(req) => ...`), per-HTTP method maps (`{ GET: ..., POST: ... }`), wildcard catch-alls (`/api/*`), redirects (`responseRedirect`), and lazy static file serving (`file`).
- **Programmatic Builder (`App`)**: `App` class supporting method registrations (`app.get()`, `app.post()`, `app.put()`, `app.delete()`, `app.patch()`, `app.route()`), `app.launch()`, and `app.stop()`.
- **Response Builders**: Free `#[napi]` functions (`response`, `responseJson`, `responseRedirect`).

### 2. Configuration Subsystem (`src/config.rs`)

- **`Config` Class**: Static constructors (`debugDefault`, `releaseDefault`, `defaultConfig`, `figment`, `tryFrom`, `from`).
- **Getters & Setters**: Complete parameter support for `address`, `port`, `workers`, `maxBlocking`, `ident`, `ipHeader`, `limits`, `tempDir`, `keepAlive`, `tls`, `secretKey`, `shutdown`, `logLevel`, and `cliColors`.

### 3. Route Subsystem (`src/route.rs`)

- **`Route` Class**: Constructors (`Route.new`, `Route.ranked`), property getters (`name`, `method`, `uri`, `rank`, `format`).
- **Default Ranking**: Replicates Rocket's default ranking table rules (`-12` through `-1`).

### 4. Route URI Subsystem (`src/route_uri.rs`)

- **`RouteUri` Class**: Mirror of Rocket's `RouteUri` struct (`base`, `unmountedOrigin`, `origin`).
- **Methods**: `path()`, `query()`, `asStr()`, and `defaultRank()`.

### 5. Custom Handler Subsystem (`src/handler.rs`)

- **`Handler` Class**: Wraps JS callback functions implementing Rocket's `Handler` trait.
- **`Outcome` Builders**: `outcomeSuccess()`, `outcomeError()`, `outcomeForward()`.

### 6. Code Generation & Decorators (`src/codegen.rs`)

- **Pattern Translation**: Translates Express/Bun patterns (`:id`, `*`) into Rocket syntax (`<id>`, `<_..>`).
- **Route URI Parsing**: `parseRouteUri` and `parseRocketRoutePattern`.
- **Route Decorators**: `@get`, `@post`, `@put`, `@del`, `@patch`, `@head`, `@options`, `@route`, `@catch_`.
- **SDK & Type Generators**: `generateClient`, `generateTypes`, `generateModule`, `generateTypescriptClient`, `generateTypescriptInterface`.

### 7. Server Lifecycle (`src/main.rs`)

- **`Rocket` Class**: `new(config)`, `mount`, `register`, `launch`, `shutdown`.

---

## Test Verification

All unit test suites in `src/*.test.ts` pass cleanly:

- `src/server.test.ts`
- `src/codegen.test.ts`
- `src/config.test.ts`
- `src/route.test.ts`
- `src/route-uri.test.ts`
- `src/handler.test.ts`
- `src/main.test.ts`

Total: 22/22 unit tests passing.

---

## Future Roadmap

1. **Streaming Responses**: Enable readable stream responses over the NAPI threadsafe function bridge.
2. **Rocket Fairings**: Expose Rocket `Fairing` hook bindings (`on_request`, `on_response`, `on_shutdown`) to TypeScript.

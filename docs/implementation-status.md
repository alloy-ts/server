# Implementation Status: Rocket Web Framework NAPI Bindings

This document provides a technical status summary of the Rocket web framework NAPI bindings implemented for `@alloy-ts/server`.

## Architecture & Native Modules

The library is organized into six main Rust modules exposed via NAPI-RS (`src/lib.rs`):

| Module    | Location         | Purpose                                                 | Key NAPI Exports                                                                                                       |
| --------- | ---------------- | ------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| `server`  | `src/server.rs`  | HTTP Server runtime & `Alloy.serve` compatibility       | `serve`, `Server`, `file`, `response`, `responseJson`, `responseRedirect`, `registerRoute`, `RequestInput`, `Response` |
| `config`  | `src/config.rs`  | Rocket server configuration & Figment provider bindings | `Config`, `ConfigInit`, `Limits`, `Shutdown`, `TlsConfig`, `MutualTls`, Profile constants                              |
| `codegen` | `src/codegen.rs` | Macro and code-generation runtime bridges               | `uri()`                                                                                                                |
| `app`     | `src/app.rs`     | Programmatic Rocket application lifecycle & `App`       | `App`, `Rocket`, `RocketConfig`, `launch`, `rocketMain`, `RequestPayload`, `ResponsePayload`                           |
| `handler` | `src/handler.rs` | Rocket `Handler` trait bindings & Outcome types         | `HandlerOutcome`, `OutcomeKind`, `JsHandler`                                                                           |
| `route`   | `src/route.rs`   | Route metadata, rank computation & ranking table        | `RouteInfo`, `computeDefaultRank`, `createRouteInfo`                                                                   |

## Implementation Review: Rocket Working in TS

- **Does Rocket work properly in TypeScript / Node.js?**
  Yes! Rocket operates as a native multi-threaded HTTP web server. JavaScript route handlers are invoked via threadsafe NAPI callbacks (`ThreadsafeFunction`), receiving strongly-typed request payloads (`RequestInput`, `RequestPayload`) and resolving `Response` objects.
- **Is `dist/index.js` 100% auto-generated?**
  Yes! All JavaScript/TypeScript bindings and `.d.ts` declaration files are generated directly from `#[napi]` macro exports in Rust via `@napi-rs/cli`.

## Supported Features

- [x] **High-Performance Tokio Runtime Integration**: Rocket server launched on dedicated background multi-threaded Tokio runtime threads.
- [x] **Route Table Parsing**:
  - Static route strings (`/api/status`).
  - Dynamic path parameters (`:param` and `<param>`) mapped into `req.params`.
  - Per-HTTP method handler maps (`GET`, `POST`, `PUT`, `DELETE`, `PATCH`, `HEAD`, `OPTIONS`).
  - Multi-segment glob/wildcard routes (`/api/*` mapped to Rocket's `/<_..>`).
- [x] **Application Builder (`App`)**:
  - Direct builder pattern class in Rust (`App`) with `app.get()`, `app.post()`, `app.put()`, `app.delete()`, `app.patch()`, `app.launch()`, `app.stop()`.
- [x] **Response Builders**:
  - `response(body, init)` for text/raw responses.
  - `responseJson(data, init)` with automatic JSON serialization & `application/json` header.
  - `responseRedirect(url, status)` for HTTP redirects.
  - `file(path)` for lazy file serving.
- [x] **Configuration System (`Config`)**:
  - Static profile constructors (`debugDefault()`, `releaseDefault()`, `defaultConfig()`, `figment()`).
  - Partial override constructors (`tryFrom()`, `from()`).
  - Real-time property getters/setters (address, port, workers, maxBlocking, keepAlive, ident, ipHeader, limits, shutdown, logLevel, cliColors, tls, secretKey).
- [x] **Codegen Helpers**:
  - `uri(pattern, params, query)` for type-safe and URL-safe parameter interpolation and percent encoding.
- [x] **Handler Trait & Outcome Bindings (`handler`)**:
  - Bridge JS callbacks to Rocket's `Handler` trait.
  - `HandlerOutcome` and `OutcomeKind` (`Success`, `Error`, `Forward`) for explicit outcome handling.
- [x] **Route Metadata & Ranking Table (`route`)**:
  - `RouteInfo` and `computeDefaultRank()` matching Rocket's default ranking table (-12 static to -1 wild).

## Unit Testing & Verification

- **Tests Executed**: `vp test` (13/13 unit tests passing across `src/main.test.ts`, `src/config.test.ts`, and `src/handler.test.ts`).
- **Code Check**: `vp check` passing with zero warnings or errors.
- **Example Runner**: Example TS scripts verified via `@oxc-node/core/register.mjs`.

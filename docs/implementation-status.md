# Implementation Status: Rocket Web Framework NAPI Bindings

This document provides a technical status summary of the Rocket web framework NAPI bindings implemented for `@alloy-ts/server`.

## Architecture & Native Modules

The library is organized into four main Rust modules exposed via NAPI-RS (`src/lib.rs`):

| Module    | Location         | Purpose                                                 | Key NAPI Exports                                                                                                       |
| --------- | ---------------- | ------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| `server`  | `src/server.rs`  | HTTP Server runtime & `Alloy.serve` compatibility       | `serve`, `Server`, `file`, `response`, `responseJson`, `responseRedirect`, `registerRoute`, `RequestInput`, `Response` |
| `config`  | `src/config.rs`  | Rocket server configuration & Figment provider bindings | `Config`, `ConfigInit`, `Limits`, `Shutdown`, `TlsConfig`, `MutualTls`, Profile constants                              |
| `codegen` | `src/codegen.rs` | Macro and code-generation runtime bridges               | `uri()`                                                                                                                |
| `app`     | `src/app.rs`     | Programmatic Rocket application lifecycle               | `Rocket`, `RocketConfig`, `launch`, `rocketMain`, `RequestPayload`, `ResponsePayload`                                  |

## Supported Features

- [x] **High-Performance Tokio Runtime Integration**: Rocket server launched on dedicated background multi-threaded Tokio runtime threads.
- [x] **Route Table Parsing**:
  - Static route strings (`/api/status`).
  - Dynamic path parameters (`:param` and `<param>`) mapped into `req.params`.
  - Per-HTTP method handler maps (`GET`, `POST`, `PUT`, `DELETE`, `PATCH`, `HEAD`, `OPTIONS`).
  - Multi-segment glob/wildcard routes (`/api/*` mapped to Rocket's `/<_..>`).
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

## Unit Testing & Verification

- **Tests Executed**: `vp test` (9/9 unit tests passing across `src/main.test.ts` and `src/config.test.ts`).
- **Code Check**: `vp check` & `vp fmt` passing with zero warnings or errors.
- **Example Runner**: Example TS scripts verified via `@oxc-node/core/register.mjs`.

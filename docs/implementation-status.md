# Implementation Status: Rocket NAPI Bindings & Codegen

This document tracks the implementation status of NAPI bindings between the Rocket web framework (v0.5.1) and TypeScript (Node.js).

---

## Overview

| Component                           | Module           | Status      | Notes                                                                                          |
| :---------------------------------- | :--------------- | :---------- | :--------------------------------------------------------------------------------------------- |
| **HTTP Server (`Alloy.serve`)**     | `src/server.rs`  | ✅ Complete | Bun/Alloy-style route table mounting, static files, response helpers, and fallback handling.   |
| **Rocket Lifecycle (`Rocket`)**     | `src/rocket.rs`  | ✅ Complete | Programmatic mounting, catcher registration, ignition, liftoff, and graceful shutdown.         |
| **Route Metadata (`Route`)**        | `src/route.rs`   | ✅ Complete | NAPI class for `rocket::Route` supporting rank computation, `create`, `ranked`, and `mapBase`. |
| **Configuration System (`Config`)** | `src/config.rs`  | ✅ Complete | Full mirror of `rocket::Config` with Figment support, limits, and profile defaults.            |
| **Route Pattern Parser**            | `src/codegen.rs` | ✅ Complete | Extracts dynamic path and query parameters (`parseRocketRoutePattern`).                        |
| **Pattern Translation**             | `src/codegen.rs` | ✅ Complete | Converts Rocket route patterns to Alloy/Bun patterns (`translatePatternToAlloy`).              |
| **Client SDK Generator**            | `src/codegen.rs` | ✅ Complete | Generates typed TypeScript `fetch` client SDK classes (`generateTypescriptClient`).            |
| **Route Handler Generator**         | `src/codegen.rs` | ✅ Complete | Generates TypeScript route handler boilerplate (`generateRouteHandler`).                       |
| **Type Interface Generator**        | `src/codegen.rs` | ✅ Complete | Converts field definitions into TypeScript interfaces (`generateTypescriptInterface`).         |

---

## Exported TypeScript Surface

### Classes & Constructors

- `Server` — Handle returned by `serve` exposing `url`, `port`, and `stop()`.
- `Rocket` — Class wrapping `rocket::Rocket<Build>` with `mount`, `register`, `launch`, and `shutdown`.
- `Route` — NAPI class mirroring `rocket::Route` with `create`, `ranked`, and `mapBase`.
- `Config` — Mirror of `rocket::Config` with static factories `debugDefault()`, `releaseDefault()`, `defaultConfig()`, `figment()`, `tryFrom()`, and `from()`.

### Free Functions & Helpers

- `serve(routes, fetch, options)` — Initialize and launch HTTP server.
- `launch(config)` / `rocketMain(config)` — Launch Rocket server using configuration provider.
- `response(body, init)` / `responseJson(data, init)` / `responseRedirect(url, status)` — Response constructors.
- `reqText(req)` / `reqJson(req)` — Request body parsing helpers.
- `file(path)` — Static file marker.
- `registerRoute(method, path, handler)` — Low-level route registration.
- `parseRocketRoutePattern(pattern)` — Route parameter extraction.
- `translatePatternToAlloy(pattern)` — Pattern syntax translation.
- `generateTypescriptClient(routes, options)` — Client SDK generation.
- `generateRouteHandler(route)` — Handler boilerplate generation.
- `generateTypescriptInterface(name, fields)` — Interface generation.

---

## Verification & Testing

- **Vitest Unit Tests**: `src/main.test.ts`, `src/codegen.test.ts`, `src/server.test.ts`, `src/route.test.ts`
- **Node CJS Runner**: `test.cjs`
- **Linting & Formatting**: `npm run check`, `npm run fmt`

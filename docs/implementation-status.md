# Rocket NAPI Bindings - Implementation Status Report

## Executive Summary

The `@alloy-ts/server` native bindings provide a high-performance bridge between the **Rocket web framework** (Rust) and **TypeScript / Node.js**. The bindings enable full Rocket web server performance while presenting clean, familiar TypeScript APIs (including `Bun.serve` compatibility, Rocket's `Route`, `RouteUri`, and `Handler` structs, and Rocket's `App` builder pattern).

---

## Implemented NAPI Binding Architecture

| Module / Component  | TypeScript API                                             | Rust Backend (`src/*.rs`)                | Status      |
| :------------------ | :--------------------------------------------------------- | :--------------------------------------- | :---------- |
| **Server Engine**   | `Alloy.serve(options)` / `serve()`                         | `src/server.rs` (`server_serve_options`) | ✅ Complete |
| **App Builder**     | `new App()`, `app.get()`, `app.post()`, etc.               | `src/server.rs` (`App`, `AppRoute`)      | ✅ Complete |
| **Route Struct**    | `Route`, `Route.ranked()`, `Route.defaultRank()`           | `src/route.rs` (`Route`)                 | ✅ Complete |
| **RouteUri Struct** | `RouteUri`, `RouteUri.parse()`                             | `src/route_uri.rs` (`RouteUri`)          | ✅ Complete |
| **Handler Struct**  | `Handler`, `handler.isValid()`                             | `src/handler.rs` (`Handler`)             | ✅ Complete |
| **Response Class**  | `new Response()`, `Response.json()`, `Response.redirect()` | `src/server.rs` (`ResponseClass`)        | ✅ Complete |
| **File Serving**    | `Alloy.file(path)`                                         | `src/server.rs` (`AlloyFile`)            | ✅ Complete |
| **Configuration**   | `Config`, `ConfigInit`                                     | `src/config.rs` (`Config`, `ConfigInit`) | ✅ Complete |
| **Code Generation** | `parseRoutes`, `generateClient`, `generateTypes`           | `src/codegen.rs`                         | ✅ Complete |

---

## Detailed Feature Coverage

### 1. `Alloy.serve` (Bun.serve Compatibility)

- **Single-Object Configuration**: Accepts `{ routes, fetch, port, hostname, unix }`.
- **Environment Fallbacks**: Respects `$BUN_PORT`, `$PORT`, `$NODE_PORT`, defaulting to `3000`.
- **Route Matching**:
  - **Static Paths**: e.g. `/api/status` -> Rocket route `/api/status`.
  - **Dynamic Parameters**: `/users/:id` -> Rocket route `/users/<id>`, extracted into `req.params`.
  - **Wildcard Catch-alls**: `/api/*` -> Rocket route `/api/<_..>` with dynamic rank calculation (rank 8 for `/api/*`, rank 100 for `/*`) preventing route collisions.
  - **Method Maps**: `{ GET: fn, POST: fn }` mapping multiple HTTP handlers to a single path.
  - **Redirects & File Responses**: Native support for `Response.redirect()` and `Alloy.file()`.
  - **Fallback Handler**: Optional `fetch(req)` fallback for unmatched routes.

### 2. Rocket `Route`, `RouteUri`, & `Handler` Classes

- NAPI wrapper around Rocket's `Route` struct exposing `method`, `uri`, `rank`, `format`, and `name`.
- `RouteUri` struct exposing mount `base`, unmounted path, canonical `path`, and query string.
- `Route.defaultRank(uri)` calculates default precedence ranks according to Rocket's 12-rank color matrix (-12 to -1).
- `Handler` NAPI class wrapping JS callback functions.

### 3. Async Callback Interoperability

- JS handlers (both synchronous `(req) => Response` and asynchronous `async (req) => Response`) are wrapped in an async wrapper returning Promises.
- Invoked thread-safely from Rocket's multi-threaded Tokio async workers via `ThreadsafeFunction`.

### 4. Server Configuration (`Config`)

- Complete NAPI exposure of Rocket's Figment configuration properties:
  - `port`, `address`, `workers`, `max_blocking`, `ident`, `ip_header`, `limits`, `temp_dir`, `keep_alive`, `shutdown`, `log_level`, `cli_colors`.
- Constructors for `Config.debug()` and `Config.release()`.

---

## Verification & Test Results

- **Rust Unit Tests**: `cargo test` passes 8/8 tests.
- **Vitest Suites**: `npm test` passes 23/23 tests across `src/server.test.ts`, `src/config.test.ts`, `src/route.test.ts`, `src/route_uri.test.ts`, `src/handler.test.ts`, and `src/codegen.test.ts`.
- **Linting & Formatting**: `npm run check` (`vp check`) and `npm run fmt` (`vp fmt`) pass with 0 warnings or errors.
- **Example Execution**: All 8 TypeScript files in `examples/` verified runnable with `node --import @oxc-node/core`.

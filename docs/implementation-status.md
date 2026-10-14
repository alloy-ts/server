# Rocket NAPI Bindings — Implementation Status & Architecture Review

This document provides a technical status review and architectural breakdown of the Node.js (TypeScript) NAPI bindings for the Rocket web framework (`@alloy-ts/server` / `alloy-server-native`).

---

## Architecture Overview

```
+-------------------------------------------------------------------+
|                  TypeScript / Node.js Layer                       |
|   Alloy.serve() / App / Handler / response() / Config / Codegen   |
+-------------------------------------------------------------------+
                                  |
                         NAPI-RS ABI Bridge
                                  |
+-------------------------------------------------------------------+
|                     Rust NAPI Bindings Layer                      |
|   src/server.rs  |  src/config.rs  |  src/handler.rs  | codegen.rs  |
+-------------------------------------------------------------------+
                                  |
                        Tokio Runtime Thread
                                  |
+-------------------------------------------------------------------+
|                    Rocket 0.5 Web Framework                       |
|   rocket::build() -> mount() -> launch() -> shutdown()            |
+-------------------------------------------------------------------+
```

### Key Architectural Mechanisms

1. **Background Async Runtime Execution**:
   - `Alloy.serve()` and `Rocket.launch()` execute Rocket inside a dedicated, multi-threaded Tokio runtime running in a background OS thread.
   - The Node.js event loop remains unblocked while Rocket processes HTTP requests concurrently across thread pools.

2. **Sync/Async JS Handler Bridge (`Either<Response, Promise<Response>>`)**:
   - JS route handlers are wrapped in NAPI `ThreadsafeFunction` handles.
   - Using `Either<Response, Promise<Response>>` enables direct return values for synchronous callbacks (`(req) => response("OK")`) and awaited promises for async callbacks (`async (req) => responseJson({...})`).

3. **Handler Trait Bindings**:
   - `Handler`, `HandlerRequest`, `HandlerResponse`, and outcome helpers (`createHandlerOutcomeSuccess`, `createHandlerOutcomeError`, `createHandlerOutcomeForward`) mirror Rocket's `Handler` trait abstractions for custom NAPI route handlers.

4. **Route Translation & Pattern Normalization**:
   - Bun/Express style route patterns (`/users/:id`, `/files/*`) are translated to Rocket URIs (`/users/<id>`, `/files/<_..>`).
   - Every route pattern is normalized to guarantee leading `/` prefixes required by Rocket's router.

5. **Figment Configuration Integration**:
   - Rocket's Figment configuration provider is mirrored via `Config` in `src/config.rs`.
   - Supports Rocket defaults, profile selection (`debug`, `release`), environment variables (`ROCKET_*`), and explicit partial overrides (`Config.tryFrom`).

---

## Module Status Matrix

| Module              | Native Source      | JS Export                                                                                                         | Status       | Test Suite                        |
| ------------------- | ------------------ | ----------------------------------------------------------------------------------------------------------------- | ------------ | --------------------------------- |
| **Server Engine**   | `src/server.rs`    | `Alloy.serve`, `serve`, `Server`, `file`, `response`, `responseJson`, `responseRedirect`, `registerRoute`         | **Complete** | `src/server.test.ts` (9 tests)    |
| **Config Engine**   | `src/config.rs`    | `Config`, `ConfigInit`, `Limits`, `Shutdown`, parameter constants                                                 | **Complete** | `src/config.test.ts` (5 tests)    |
| **Codegen Engine**  | `src/codegen.rs`   | `httpMethods`, `parseRoute`, `parseRoutes`, `toRocketPath`, `generateTypes`, `generateClient`, `generateModule`   | **Complete** | `src/codegen.test.ts` (11 tests)  |
| **Route Engine**    | `src/route.rs`     | `Route` class (`new`, `ranked`, `mapBase`, getters/setters)                                                       | **Complete** | `src/route.test.ts` (4 tests)     |
| **RouteUri Engine** | `src/route_uri.rs` | `RouteUri` class (`new`, `base`, `unmountedOrigin`, `origin`, `path`, `query`, `asStr`, `defaultRank`, `mapBase`) | **Complete** | `src/route_uri.test.ts` (3 tests) |
| **Handler Engine**  | `src/handler.rs`   | `Handler`, `HandlerOutcomeKind`, `createHandlerOutcomeSuccess`, `createHandlerOutcomeError`, etc.                 | **Complete** | `src/handler.test.ts` (4 tests)   |
| **App Builder**     | `src/main.ts`      | `App` class (`get`, `post`, `put`, `delete`, `patch`, `head`, `options`, `route`, `launch`, `stop`)               | **Complete** | `src/server.test.ts`              |

---

## Examples & Demos

| Example File             | Description                                                                                                          | Execution Test |
| ------------------------ | -------------------------------------------------------------------------------------------------------------------- | -------------- |
| `examples/server.ts`     | Demonstrates `Alloy.serve` with static/dynamic routes, method maps, wildcards, redirects, files, and fallback fetch. | **Verified**   |
| `examples/app-routes.ts` | Translates canonical Rocket quickstart example into TypeScript `App` + `response`.                                   | **Verified**   |
| `examples/codegen.ts`    | Demonstrates route parsing, type generation, client SDK generation, and module generation.                           | **Verified**   |
| `examples/config.ts`     | Demonstrates Rocket `Config` creation, field mutation, overrides, and launching configured server.                   | **Verified**   |
| `examples/rest-api.ts`   | Full CRUD REST API for user management with JSON responses and path parameters.                                      | **Verified**   |

---

## Test & Verification Summary

- **Rust Unit Tests (`cargo test`)**: 8/8 tests passing.
- **TypeScript Unit Tests (`vp test`)**: 36/36 tests passing across `src/codegen.test.ts`, `src/config.test.ts`, `src/handler.test.ts`, `src/route.test.ts`, `src/route_uri.test.ts`, and `src/server.test.ts`.
- **Type Checking & Formatting (`vp check`)**: 0 warnings, 0 lint errors, 0 type errors across all files.
- **OxNode Execution**: Tested and verified with `@oxc-node/core` and Node experimental type stripping.

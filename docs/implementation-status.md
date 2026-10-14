# Rocket Web Framework for TypeScript - Implementation Status

This document summarizes the implementation status of the Node.js NAPI bindings for the Rust Rocket web framework (`@alloy-ts/server`).

---

## Overview

| Feature Area                  |   Status    | Rocket v0.5 Module Mapping         | NAPI / TS API                                                     |       Unit Tests        |
| :---------------------------- | :---------: | :--------------------------------- | :---------------------------------------------------------------- | :---------------------: |
| **Server Lifecycle & Launch** | ✅ Complete | `rocket::Rocket`, `rocket::ignite` | `Alloy.serve`, `Alloy.app`, `Server`                              |  `src/server.test.ts`   |
| **Route Processing**          | ✅ Complete | `rocket::Route`                    | `Route`, `Method`                                                 |   `src/route.test.ts`   |
| **Route URI Processing**      | ✅ Complete | `rocket::route::RouteUri`          | `RouteUri`                                                        | `src/route_uri.test.ts` |
| **Handlers**                  | ✅ Complete | `rocket::route::Handler`           | `Handler`                                                         |  `src/handler.test.ts`  |
| **Catchers & Error Handling** | ✅ Complete | `rocket::catcher::Catcher`         | `Catcher`                                                         |  `src/catcher.test.ts`  |
| **Configuration**             | ✅ Complete | `rocket::Config`                   | `Config`, `Profile`                                               |  `src/config.test.ts`   |
| **Code Generation**           | ✅ Complete | `rocket_codegen`                   | `parseRoute`, `generateTypes`, `generateClient`, `generateModule` |  `src/codegen.test.ts`  |

---

## Detailed Component Status

### 1. Server & Application Builder (`src/server.rs`, `src/server.test.ts`)

- **Single Options Overload & Positional Args:** `Alloy.serve({ routes, fetch, port, hostname, unix })` and `Alloy.serve(routes, fetch, port, hostname)`.
- **Port Allocation:** Full support for `port: 0` dynamic binding synchronized via Rocket liftoff fairing MPSC channels. Supports environment variable fallbacks (`BUN_PORT`, `PORT`, `NODE_PORT`).
- **Unix Sockets:** Full support for Unix domain socket binding (`unix: "/path/to/socket"`).
- **Fluent App Builder:** `Alloy.app()` supporting `.get()`, `.post()`, `.put()`, `.delete()`, `.catch()`, `.launch()`, and `.stop()`.

### 2. Code Generation (`src/codegen.rs`, `src/codegen.test.ts`)

- **Route Parsing:** `parseRoute` and `parseRoutes` convert Rocket route signatures (`GET /users/<id>?<details..>`) into structured Route ASTs.
- **Path Translation:** `toRocketPath` converts express/colon path parameters (`/users/:id`) to Rocket format (`/users/<id>`).
- **HTTP Methods:** `httpMethods()` exposes standard Rocket HTTP methods.
- **CodeGen Generators:**
  - `generateTypes`: Generates TypeScript interfaces and type definitions for route params/queries.
  - `generateClient`: Generates type-safe fetch client functions for routes.
  - `generateModule`: Generates complete TypeScript module exports.

### 3. Configuration (`src/config.rs`, `src/config.test.ts`)

- **Config Class:** NAPI bindings for `rocket::Config`.
- **Profiles:** `Profile.Debug`, `Profile.Release`.
- **Getters & Setters:** `address`, `port`, `workers`, `keepAlive`, `logLevel`, `secretKey`, `tempDir`, `ident`.
- **Validation:** `config.extract()` for eager validation.

### 4. Routes (`src/route.rs`, `src/route.test.ts`)

- **Route Class:** Wraps `rocket::Route`.
- **Rank Computation:** Accurate calculation of Rocket default route ranking table values (`-12` to `-1`).
- **Base Path Mapping:** `route.mapBase(base)` to rebase route mounts.

### 5. Route URIs (`src/route_uri.rs`, `src/route_uri.test.ts`)

- **RouteUri Class:** Wraps `rocket::route::RouteUri`.
- **Properties:** `base`, `path`, `query`, `asStr`, `unmountedOrigin`.
- **Normalization:** Enforces leading slashes for mounted URI paths.

### 6. Handlers & Catchers (`src/handler.rs`, `src/handler.test.ts`, `src/catcher.rs`, `src/catcher.test.ts`)

- **Handler:** JS callback wrapping with support for status codes, headers, text, and JSON bodies.
- **Catcher:** Handles status codes in `[400, 600)` range and default catchers.

---

## Test Verification

All 54 TypeScript integration and unit test cases pass cleanly across the 7 test suites:

- `src/codegen.test.ts`
- `src/server.test.ts`
- `src/config.test.ts`
- `src/route.test.ts`
- `src/route_uri.test.ts`
- `src/handler.test.ts`
- `src/catcher.test.ts`

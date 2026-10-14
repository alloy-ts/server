# Rocket Web Framework NAPI Bindings - Implementation Status

This document summarizes the current implementation status, supported features, and future roadmap for the Rocket web framework Node.js / TypeScript NAPI bindings (`@alloy-ts/server`).

---

## Current Status Overview

| Component                               | Status      | Description                                                                                            |
| :-------------------------------------- | :---------- | :----------------------------------------------------------------------------------------------------- |
| **Rocket Core Server Engine**           | ✅ Complete | Runs Rocket 0.5 on a Tokio runtime in background threads.                                              |
| **`Alloy.serve()` Server API**          | ✅ Complete | Full Bun/Alloy compatible route table parsing (`/api/status`, `:id`, `{ GET, POST }`, `*`, redirects). |
| **`App` Builder API**                   | ✅ Complete | Method-chaining builder interface (`app.get()`, `app.post()`, `app.launch()`).                         |
| **Request / Response Wire Translation** | ✅ Complete | Full headers, status codes, query strings, path params, and bodies across NAPI boundary.               |
| **Static File Serving**                 | ✅ Complete | `Alloy.file(path)` lazy loading file response marker.                                                  |
| **TypeScript Codegen Decorators**       | ✅ Complete | `@Get`, `@Post`, `@Put`, `@Delete`, `@Patch`, `@Head`, `@Options`, `@Route`, `@Catch`.                 |
| **URI Builder (`uri()`)**               | ✅ Complete | Replaces `:name`, `<name>`, `<name..>` path variables with URL-encoded values.                         |
| **Server Configuration (`Config`)**     | ✅ Complete | `Config.debug_default()`, `Config.release_default()`, `Config.try_from()`, `Limits`, `Shutdown`.       |
| **TypeScript Execution Support**        | ✅ Complete | Compatible with `@oxc-node/core`, Node.js ESM/CJS, and `vite-plus`.                                    |

---

## Detailed Implementation Checklist

### 1. HTTP Server Runtime (`src/server.rs`)

- [x] Spawns Rocket multi-threaded Tokio runtime on background OS threads.
- [x] Ephemeral port selection (`port: 0`) with automatic local socket binding.
- [x] Graceful shutdown handle via `Server.stop()`.
- [x] Rocket `Fairing` on liftoff capturing actual bound port & shutdown token.
- [x] Custom Rocket request dispatcher and handler bridging to JS thread-safe functions (`ThreadsafeFunction`).

### 2. Rocket Configuration (`src/config.rs`)

- [x] `Config` class wrapping `rocket::config::Config`.
- [x] Static factory methods (`debug_default`, `release_default`, `default_config`, `figment`, `try_from`).
- [x] Support for limits (`Limits`) and shutdown options (`Shutdown`).
- [x] Getters and setters for `address`, `port`, `workers`, `maxBlocking`, `ident`, `logLevel`, etc.

### 3. TypeScript / JS Public Surface (`src/main.ts`)

- [x] `Alloy` namespace containing `serve`, `file`, `launch`, `Response`, `Config`, `Server`.
- [x] `App` class for `app.get()`, `app.post()`, `app.launch()`.
- [x] `response(body, init)` helper.
- [x] `uri(pattern, params)` builder function.
- [x] Route decorators (`@Get`, `@Post`, `@Put`, `@Delete`, `@Patch`, `@Head`, `@Options`, `@Route`, `@Catch`).

---

## Future Roadmap & Missing Items

The following features are planned for future iterations:

1. **Response Stream Chunking**:
   - Support streaming async iterators / readable streams for large payloads or SSE (Server-Sent Events).
2. **WebSocket & HTTP/2 Protocols**:
   - Bridge Rocket WebSocket connections and HTTP/2 multiplexing options to JS event handlers.
3. **Multipart Form Guard Parser**:
   - Provide high-level TypeScript request guards for file uploads and multipart forms.
4. **JS Fairings (Middleware)**:
   - Allow custom JavaScript fairings (`on_request`, `on_response`, `on_shutdown`) to attach to Rocket lifecycle events.

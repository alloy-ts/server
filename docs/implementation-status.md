# Rocket Web Framework NAPI Bindings - Implementation Status

This document summarizes the current implementation status, supported features, and future roadmap for the Rocket web framework Node.js / TypeScript NAPI bindings (`@alloy-ts/server`).

---

## Current Status Overview

| Component                                | Status      | Description                                                                                      |
| :--------------------------------------- | :---------- | :----------------------------------------------------------------------------------------------- |
| **Rocket Core Server Engine**            | ✅ Complete | Runs Rocket 0.5 on a Tokio runtime in background threads.                                        |
| **`serve()` Server API**                 | ✅ Complete | Full route table parsing (`/api/status`, `:id`, `{ GET, POST }`, redirects).                     |
| **`Rocket` Class API**                   | ✅ Complete | `Rocket` builder interface (`rocket.mount()`, `rocket.register()`, `rocket.launch()`).           |
| **Request / Response Wire Translation**  | ✅ Complete | Full headers, status codes, query strings, path params, and bodies across NAPI boundary.         |
| **Static File Serving**                  | ✅ Complete | `file(path)` lazy loading file response marker.                                                  |
| **Route Registration (`registerRoute`)** | ✅ Complete | Direct route registration for HTTP methods (`GET`, `POST`, `PUT`, `DELETE`, etc.).               |
| **URI Builder (`uri()`)**                | ✅ Complete | Replaces `:name`, `<name>`, `<name..>` path variables with URL-encoded values.                   |
| **Server Configuration (`Config`)**      | ✅ Complete | `Config.debug_default()`, `Config.release_default()`, `Config.try_from()`, `Limits`, `Shutdown`. |
| **Request & Route Utilities**            | ✅ Complete | `Request`, `Route`, `Handler`, `RouteUri` wrappers exposing Rocket data structures.              |
| **TypeScript Execution Support**         | ✅ Complete | Compatible with `@oxc-node/core`, Node.js ESM/CJS, and `vite-plus`.                              |

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

### 3. Native NAPI Surface (`src/app.rs`, `src/route.rs`, `src/handler.rs`, `src/request.rs`, `src/route_uri.rs`)

- [x] Native `Rocket` class for mounting routes, registering catchers, and launching the server.
- [x] Native `serve()` function for serving route tables.
- [x] Native `Route`, `Handler`, `Request`, `RouteUri` objects.
- [x] Native `response(body, init)` helper.
- [x] Native `uri(pattern, params)` builder function.

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

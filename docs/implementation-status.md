# Implementation Status & Roadmap

This document outlines the current implementation status of Rocket NAPI bindings for Node.js / TypeScript, identifies missing features, and sets out the roadmap for completion.

---

## Current Implementation Status

### 1. Server Lifecycle & Routing (`src/server.rs` & `src/main.ts`)

| Feature                  |    Status     | Details                                                                          |
| :----------------------- | :-----------: | :------------------------------------------------------------------------------- |
| `Alloy.serve()`          | **Supported** | Mounts route table onto Rocket's multi-threaded Tokio runtime.                   |
| Static & Dynamic Routes  | **Supported** | Supports exact paths, Express/Bun style `:param`, and Rocket `<param>` syntax.   |
| Per-Method Handler Maps  | **Supported** | Supports `{ GET: fn, POST: fn }` handler objects per route pattern.              |
| Wildcards & Catch-Alls   | **Supported** | Supports `/api/*` translation to Rocket's `/<_..>` catch-all pattern.            |
| Redirects                | **Supported** | `responseRedirect(url, status)` via `Location` header.                           |
| Static File Serving      | **Supported** | `Alloy.file(path)` asynchronously reads and serves files from disk.              |
| `App` Builder Class      | **Supported** | 1:1 runtime counterpart for Rocket's `rocket::build().mount("/", routes![...])`. |
| Server Graceful Shutdown | **Supported** | `server.stop()` triggers Rocket shutdown notification.                           |

### 2. Configuration (`src/config.rs`)

| Feature                       |    Status     | Details                                                                               |
| :---------------------------- | :-----------: | :------------------------------------------------------------------------------------ |
| `Config.debugDefault()`       | **Supported** | Exposes Rocket debug default configuration.                                           |
| `Config.releaseDefault()`     | **Supported** | Exposes Rocket release default configuration.                                         |
| Property Mutators             | **Supported** | Getters and setters for `port`, `address`, `workers`, `maxBlocking`, `logLevel`, etc. |
| Partial Overrides (`tryFrom`) | **Supported** | Merges partial configuration override objects.                                        |

### 3. Code Generation (`src/codegen.rs`)

| Feature              |    Status     | Details                                                                                      |
| :------------------- | :-----------: | :------------------------------------------------------------------------------------------- |
| `Codegen` NAPI Class | **Supported** | Collects route metadata (`RouteCodegenMeta`) and generates TypeScript route structure stubs. |
| `parseRoutePattern`  | **Supported** | Parses path parameters and query parameters from Rocket route string patterns.               |

---

## What is Currently Missing or Simplified?

1. **Streaming Body & Chunked Transfer**:
   - Request bodies are currently read in-memory up to a 1 MiB limit before calling JS handlers.
   - Response bodies are returned as full buffers (`Vec<u8>`). Full streaming (Server-Sent Events, WebSockets, ReadableStream) is not yet bridged.

2. **Custom Request Guards & Fairings**:
   - Rocket request guards (e.g. `FromRequest`, headers, cookies, state) are currently extracted into plain JS `req.headers` / `req.params`. Custom TS-defined request guards or Rocket fairings are not exposed.

3. **Catcher (Error Handler) Customization**:
   - Custom Rocket error catchers (`#[catch(404)]`) can be registered in low-level Rust NAPI bindings (`register`), but high-level `Alloy.serve({ error(err) { ... } })` fallback catchers are not fully exposed.

4. **TLS / Mutual TLS (mTLS) Features**:
   - Rocket's built-in TLS and mTLS features in `src/config.rs` require crate features (`tls`, `secrets`, `mtls`) which are gated.

---

## Next Steps for Finishing Rocket NAPI Bindings

1. **Phase 1: Streaming Request & Response Bodies**
   - Implement NAPI AsyncIterable / ReadableStream bridging for `req.body` and `res.body` to support large file uploads and Server-Sent Events (SSE).

2. **Phase 2: Full Catcher (Error Handler) API**
   - Add `catchers` support in `Alloy.serve` and `App` to allow handling custom HTTP status codes (400, 404, 500) with dedicated JS callbacks.

3. **Phase 3: TLS and Cookie Encryption**
   - Re-enable `secrets` and `tls` crate features in `Cargo.toml` with NAPI bindings for certificate chains and encrypted private cookie jars.

4. **Phase 4: Automatic Decorator Codegen (TS Macro/Transformer)**
   - Provide a TypeScript AST transformer or Babel/SWC plugin that reads TS decorators (`@Get("/")`) and automatically calls `Codegen` / `registerRoute`.

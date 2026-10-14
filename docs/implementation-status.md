# Implementation Status & Roadmap: Rocket NAPI Bindings

This document outlines the current implementation status, architecture, and roadmap for `@alloy-ts/server` (Rocket NAPI bindings for TypeScript).

---

## 1. Feature Support Matrix

| Feature Category | Status | Details |
|---|---|---|
| **`Alloy.serve()` Server** | **Completed** | Full support for Bun/Alloy.serve route tables: static routes, dynamic path parameters (`:id`), per-method handler maps (`{ GET, POST }`), wildcard catch-alls (`/api/*`), redirects, static files (`Alloy.file`), and fallback `fetch`. |
| **`App` Router Class** | **Completed** | Fluent routing interface (`app.get()`, `app.post()`, `app.put()`, `app.delete()`, `app.patch()`, `app.launch()`). |
| **Type-Safe URIs (`Alloy.uri`)** | **Completed** | Interpolates named dynamic path segments (`<id>` / `:id`) with RFC 3986 percent-encoding. |
| **Response Helpers** | **Completed** | `Alloy.Response`, `Response.json`, and `Response.redirect` wire-type constructors. |
| **Configuration (`Config`)** | **Completed** | Programmatic access to Rocket `Config` fields, default profile inspection (`debugDefault`, `releaseDefault`), `tryFrom` figment overrides, and parameter name listing. |
| **Rocket App Lifecycle (`Rocket`)** | **Completed** | Low-level NAPI struct (`new`, `mount`, `register`, `launch`, `shutdown`). |
| **Codegen Utilities** | **Completed** | Route attribute parser (`parseRouteAttribute`), catcher attribute parser (`parseCatcherAttribute`), and TS manifest generator (`generateTsRouteManifest`). |

---

## 2. What's Working Properly in TypeScript?

- **Rocket Web Server Integration:** Rocket launches inside a dedicated multi-threaded Tokio background thread without blocking Node.js's main thread loop.
- **Threadsafe JS Handlers:** Handler callbacks run across thread boundaries via `ThreadsafeFunction`, receiving full request metadata (`method`, `path`, `query`, `headers`, `params`, `body`) and resolving `Promise<Response>`.
- **Ephemeral Port Auto-Assignment:** Passing `port: 0` automatically queries the OS for an open port and binds Rocket accordingly.
- **Examples & Test Suite:** Examples in `examples/server.ts` and `examples/app-routes.ts` run out-of-the-box with `npx oxnode`.

---

## 3. What's Missing? (Roadmap & Next Steps)

The following features represent the next steps for complete parity with Rocket's full feature set:

1. **Fairings & Middleware Interceptors:**
   - Expose NAPI hooks for Rocket `Fairing`s (`on_request`, `on_response`, `on_shutdown`) so TypeScript middleware can inspect or modify request/response streams.

2. **Custom Request Guards & Managed State:**
   - Bridge Rocket's `FromRequest` trait to TypeScript to allow custom authentication/guard middleware functions before route handlers run.

3. **Streaming Response Bodies (Server-Sent Events / Chunked Downloads):**
   - Extend `Response` to accept `ReadableStream` or async iterators for chunked HTTP streaming and SSE support.

4. **HTML Import & Client Bundler Integration:**
   - Implement manifest loading for pre-built HTML assets (`import index from "./index.html"`).

5. **Form & Form-Field Parsing (`FromForm`):**
   - Automatically parse multipart form data and URL-encoded query/form bodies into structured JS objects.

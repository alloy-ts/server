# Rocket NAPI Implementation Status

This document summarizes the current status of NAPI bindings for the Rocket web framework in `@alloy-ts/server`.

## Implemented NAPI Surface

### Server & Routing (`src/server.rs`)
- [x] `Alloy.serve(...)` and `Bun.serve(...)` with Bun.server options (`routes`, `fetch`, `port`, `hostname`, `unix`).
- [x] Static routes, dynamic route parameters (`:id`, `<id>`), wildcard catch-alls (`/api/*` -> `<_..>`).
- [x] Per-HTTP method handlers (`GET`, `POST`, `PUT`, `DELETE`, `PATCH`, `HEAD`, `OPTIONS`).
- [x] Web response helpers (`response`, `responseJson`, `responseRedirect`).
- [x] File serving marker (`file(path)` / `Bun.file` / `Alloy.file`).
- [x] Fallback `fetch(req)` handler for unmatched routes.
- [x] `App` class for `app.get()`, `app.post()`, `app.launch()`, `app.stop()` re-expressing Rocket macros at runtime.
- [x] Per-server graceful shutdown via `server.stop()`.
- [x] Port fallback checks (`BUN_PORT`, `PORT`, `NODE_PORT`) and port `0` dynamic assignment.

### Rocket Configuration (`src/config.rs`)
- [x] Full `Config` getter/setter and associated constants.
- [x] Static factory methods (`debugDefault()`, `releaseDefault()`, `defaultConfig()`, `figment()`, `tryFrom()`, `from()`).
- [x] `Limits`, `Shutdown`, `TlsConfig`, `MutualTls` object bindings.

### Codegen & Helpers (`src/codegen.rs`)
- [x] Route definition parsing (`parseRoute`, `parseRoutes`).
- [x] TypeScript types and client generation (`generateTypes`, `generateClient`, `generateModule`).
- [x] Rocket path translator (`toRocketPath`).

---

## Missing Features / Planned Next Steps

1. **Web `Request` and `Response` Object Compatibility**:
   - Wrap NAPI responses so standard JS `new Response(...)` instances can be returned directly without needing `response(...)` wrapper helpers.
2. **Streaming & WebSockets**:
   - WebStream body support for streaming response payloads and upgrade handling for WebSockets / HTTP/2 push.
3. **Rocket Fairings and Request Guards**:
   - Expose NAPI hooks for Rocket fairings (`on_request`, `on_response`, `on_shutdown`) and managed state / request guards to JS context.
4. **Catcher Support in `Alloy.serve`**:
   - NAPI API for mounting custom error catchers (404, 500) directly into the Rocket router.

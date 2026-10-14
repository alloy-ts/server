# Rocket NAPI Implementation Status

This document summarizes the current status of NAPI bindings for the Rocket web framework in `@alloy-ts/server`.

## Implemented NAPI Surface

### Server & Routing (`src/server.rs`, `src/route.rs`, `src/route_uri.rs`, `src/handler.rs`)
- [x] `Alloy.serve(...)` and `Bun.serve(...)` with Bun.server options (`routes`, `fetch`, `port`, `hostname`, `unix`).
- [x] Static routes, dynamic route parameters (`:id`, `<id>`), wildcard catch-alls (`/api/*` -> `<_..>`).
- [x] Per-HTTP method handlers (`GET`, `POST`, `PUT`, `DELETE`, `PATCH`, `HEAD`, `OPTIONS`).
- [x] Web response helpers (`response`, `responseJson`, `responseRedirect`).
- [x] File serving marker (`file(path)` / `Bun.file` / `Alloy.file`).
- [x] Fallback `fetch(req)` handler for unmatched routes.
- [x] `App` class for `app.get()`, `app.post()`, `app.launch()`, `app.stop()` re-expressing Rocket macros at runtime.
- [x] `Route` NAPI class (`Route.ranked(rank, method, uri)`) inspecting and constructing Rocket route metadata.
- [x] `RouteUri` NAPI class (`RouteUri(base, uri)`) parsing and matching route URIs (`base()`, `path()`, `query()`, `as_str()`).
- [x] `Outcome` and `OutcomeHelper` NAPI bindings for `Success`, `Error`, and `Forward` handler outcomes.
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

1. **WebStream Body & WebSockets**:
   - Streaming response payloads using Node/Web ReadableStream and connection upgrades for WebSockets / HTTP/2 push.
2. **Rocket Fairings and Request Guards**:
   - NAPI hooks for Rocket fairings (`on_request`, `on_response`, `on_shutdown`) and managed state / request guards passed to JS context.
3. **Catcher Support**:
   - NAPI API for registering custom error catchers (404, 500) into Rocket's catcher router.

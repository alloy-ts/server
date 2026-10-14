# Rocket NAPI Implementation Status

This document provides a review and status summary of Rocket web framework NAPI bindings for TypeScript (Node.js).

## Working / Supported Features

### 1. High-Performance HTTP Server (`Alloy.serve` & `Rocket`)

- **`Alloy.serve` (Bun/Alloy compatible API):** Supports static routes, dynamic parameter routes (`:id` and `<param>`), per-method handler maps (`{ GET, POST }`), wildcard prefix routes (`/api/*` mapped to `<_..>`), redirects via `Response.redirect()`, and lazy static file serving via `Alloy.file()`.
- **Standalone Tokio Runtime:** Rocket executes on its own multi-threaded Tokio runtime in background threads, so it does not block the Node.js event loop or require a surrounding Tokio reactor.
- **Threadsafe Handler Execution:** JavaScript route handlers execute via `napi::threadsafe_function` (`ThreadsafeFunction`), supporting async JS handlers that resolve `Promise<Response>`.
- **`Rocket` Lifecycle Class:** Exposes `new Rocket(config)`, `mount(base, method, path, handler)`, `register(base, code, handler)`, `launch()`, and `shutdown()`.

### 2. Rocket Route, RouteUri & Ranking (`Route`, `RouteUri`, `computeDefaultRank`, `HandlerOutcome`)

- **`Route` NAPI Class:** Exposes `name`, `method`, `uri`, `rank`, and `format` properties, along with `new Route(method, uri, name?, format?)` and `Route.ranked(rank, method, uri, name?, format?)`.
- **`RouteUri` NAPI Class:** Exposes `base`, `unmountedOrigin`, `origin`, `path()`, `query()`, and `asStr()`.
- **Default Ranking Algorithm (`computeDefaultRank`):** Implements Rocket's exact default ranking matrix based on path/query specificity (static, partial, wild, none) ranging from `-12` to `-1`.
- **Custom Handler Outcomes (`HandlerOutcome`):** Supports `createSuccessOutcome()`, `createErrorOutcome()`, and `createForwardOutcome()`.

### 3. Full Configuration System (`Config`)

- **Rocket Parameters:** Exposes `address`, `port`, `workers`, `max_blocking`, `keep_alive`, `ident`, `ip_header`, `limits`, `temp_dir`, `log_level`, `cli_colors`, `shutdown`, `tls`, and `secret_key`.
- **Factory Helpers:** `Config.debugDefault()`, `Config.releaseDefault()`, `Config.defaultConfig()`, `Config.figment()`, and `Config.tryFrom(init)`.
- **Eager Validation:** Modifying getters/setters (`cfg.address = "127.0.0.1"`, `cfg.limits = ...`) eagerly validates inputs and surfaces clear NAPI errors on invalid IP addresses, log levels, or byte units.

### 4. Code Generation Facilities (`Codegen`)

- **Route Attribute Parser (`parseRouteAttribute`):** Parses Rocket route syntax (`#[get("/<name>?<age>", rank = 1, format = "json", data = "<form>")]`) into `RouteMeta` with path and query parameters (`ParamMeta`).
- **Catcher Attribute Parser (`parseCatchAttribute`):** Parses HTTP status codes (`404`, `500`) or `"default"`.
- **Type-Safe URI Builder (`buildUri`):** Generates URL-safe, percent-encoded URIs given a pattern and parameter values.
- **TypeScript Code Generators:**
  - `generateTsRoutes(routes)`: Emits TypeScript route table code.
  - `generateFormSchema(fields)`: Emits `FromForm` validation functions.

---

## Missing Features / Next Steps

1. **Streaming & WebSockets:** WebSocket upgrade handlers (`server.upgrade()`) and Server-Sent Events (SSE) mid-stream timeout overrides (`server.timeout()`).
2. **Fairings & Managed State:** NAPI bindings for Rocket Fairings (`AdHoc.on_request`, `AdHoc.on_response`) and managed state injection.
3. **Advanced Guards:** Automatic request guard parsing (e.g. `FromRequest` extraction for custom JS headers/cookies) beyond route parameters and body payloads.

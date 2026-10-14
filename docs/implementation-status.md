# `@alloy-ts/server` Implementation Status & Roadmap

This document outlines the current state of NAPI bindings for the **Rocket** web framework in `@alloy-ts/server`, detailing supported capabilities, active limitations, and planned next steps.

---

## 1. Supported Capabilities

### Rocket Server Lifecycle & Configuration

- **NAPI Config Bindings (`Config`)**: Complete mirror of `rocket::config::Config` (getters/setters for address, port, workers, max_blocking, ident, ip_header, limits, keep_alive, shutdown, log_level, cli_colors, tls, secret_key).
- **Figment Integration**: Ability to initialize config from Rocket defaults or figment providers (`Config.figment()`, `Config.debugDefault()`, `Config.releaseDefault()`).
- **Low-Level Rocket Lifecycle (`Rocket`)**: Programmatic control over building, mounting routes, registering catchers, igniting, launching on dedicated background Tokio runtime, and graceful shutdown.
- **Route, RouteUri & Handler Introspection (`Route`, `RouteUri`, `HandlerOutcome`)**: Full NAPI bindings for Rocket's `Route` (method, URI, default & explicit rank computation, URI base mapping), `RouteUri` (`base`, `path`, `query`, `asStr`, `isNormalized`), and `HandlerOutcome` (`handlerSuccess`, `handlerError`, `handlerForward`).

### Bun / Alloy Compatible Server API (`Alloy.serve`)

- **Single-Object & Multi-Arg Signatures**: `Alloy.serve({ routes, fetch, port, hostname, unix })` and `serve(routes, fetch, options)`.
- **Dynamic Port Selection**: Supports `port: 0` for random available port allocation, capturing actual bound address during Rocket liftoff.
- **Port Environment Variables**: Honors `BUN_PORT`, `PORT`, and `NODE_PORT` environment variable overrides.
- **Route Matching & Priority**:
  - Static routes (`/api/status`).
  - Express/Bun dynamic parameters (`/users/:id` -> `req.params.id`).
  - Per-HTTP method route maps (`{ GET: ..., POST: ... }`).
  - Wildcard routes (`/api/*`).
  - Catch-all fallback (`fetch(req)`).
- **Request Helpers**:
  - `req.method`, `req.path`, `req.query`, `req.headers`, `req.params`, `req.body`.
  - `await req.json()` (parse JSON request body).
  - `await req.text()` (read plain text request body).
- **Response Construction**:
  - `new Response(body, init)`.
  - `Response.json(data, init)` / `responseJson`.
  - `Response.redirect(url, status)` / `responseRedirect`.
  - `Bun.file(path)` / `file(path)` lazy static file response marker.

### TypeScript Code Generation (`codegen`)

- **Route Pattern Parsing**: Parsing Rocket (`<id>`, `<id..>`) and Express/Bun (`:id`, `*`) route patterns.
- **Path Pattern Translation**: `toRocketPath()` and `translatePattern()`.
- **SDK & Type Generators**: `generateTypes`, `generateClient`, `generateModule` producing typed fetch clients and route maps.

---

## 2. Current Limitations & What's Missing

1. **Streaming Request / Response Bodies**:
   - Currently, incoming request bodies and outgoing responses are buffered in memory as byte vectors or Buffers up to 10 MiB.
   - _Missing_: Full Web-standard `ReadableStream` / `WritableStream` streaming support across the NAPI boundary.

2. **WebSockets & Connection Upgrades**:
   - Rocket 0.5 supports connection upgrades (e.g. `rocket_ws` or raw I/O handlers).
   - _Missing_: NAPI bindings for connection upgrades and WebSocket handlers (`Alloy.serve({ websocket: { ... } })`).

3. **Form & Multipart Parsing Guards**:
   - Rocket has built-in `FromForm` / `TempFile` / multipart form data handling.
   - _Missing_: Convenience NAPI bindings for `req.formData()`.

4. **Cookies & Private Cookie Jar**:
   - Rocket provides `CookieJar` and encrypted private cookies (via `secrets` feature).
   - _Missing_: Direct NAPI bridge exposing `req.cookies` jar accessor/mutator methods.

5. **Mutual TLS Request Guards**:
   - `src/config.rs` supports `TlsConfig` and `MutualTls` configuration.
   - _Missing_: NAPI request guard helper to extract validated client certificates from `req`.

---

## 3. Planned Next Steps & Roadmap

### Phase 1: Request & Response Enhancement

- Implement `req.formData()` for parsing multipart/form-data requests.
- Add cookie jar support (`req.cookies.get()`, `req.cookies.set()`, `req.cookies.getPrivate()`).

### Phase 2: WebSockets & Upgrades

- Implement WebSocket support (`ws.send()`, `ws.subscribe()`, `open`, `message`, `close` handlers) backed by Rocket's upgrade handlers.

### Phase 3: Streaming Body Support

- Integrate Node.js / Web `ReadableStream` bindings for streaming large request payloads and chunked response streaming without full buffering.

### Phase 4: Middleware & Fairings Bridge

- Expose fairing hooks (`on_request`, `on_response`, `on_shutdown`) to TypeScript so custom Rocket fairings can be written in JS.

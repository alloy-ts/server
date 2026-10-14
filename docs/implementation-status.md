# Rocket NAPI Bindings — Implementation Status & Roadmap

This document summarizes the current implementation status of Rocket web framework NAPI bindings for TypeScript (`@alloy-ts/server`), identifies what works properly today, outlines what is currently missing or limited, and details the plan for future NAPI binding enhancements.

---

## Current Status Overview

| Component                                  | Status              | NAPI Surface                                                      |
| ------------------------------------------ | ------------------- | ----------------------------------------------------------------- |
| **HTTP Server Runtime (`serve`)**          | ✅ Production Ready | `serve()`, `Alloy.serve()`                                        |
| **App Builder (`App`)**                    | ✅ Production Ready | `new App()`, `app.get()`, `app.post()`, `app.launch()`            |
| **Response Helpers**                       | ✅ Production Ready | `response()`, `responseJson()`, `responseRedirect()`, `file()`    |
| **Route Registration**                     | ✅ Production Ready | `registerRoute()`                                                 |
| **Port & Env Resolution**                  | ✅ Production Ready | `port: 0`, `BUN_PORT`, `PORT`, `NODE_PORT`                        |
| **Configuration (`Config`)**               | ✅ Fully Supported  | `Config` class, `ConfigInit`, `Limits`, `Shutdown`                |
| **TypeScript Codegen**                     | ✅ Fully Supported  | `parseRoute`, `generateTypes`, `generateClient`, `generateModule` |
| **Rocket Lifecycle (`Rocket`)**            | ⚠️ Low-level API    | `Rocket.new()`, `mount()`, `register()`, `launch()`               |
| **Catchers / Error Handlers**              | ⚠️ Basic            | Default Rocket catchers / low-level `register`                    |
| **Managed State & Request Guards**         | ❌ Missing          | Not yet surfaced to JS                                            |
| **Fairings (Liftoff / Response / Attach)** | ❌ Missing          | Internal shutdown fairing only                                    |
| **Multipart & Streaming Forms**            | ❌ Limited          | Body read up to 1MB buffer                                        |
| **WebSockets & HTTP Upgrades**             | ❌ Missing          | Not yet exposed                                                   |

---

## What Works Properly

1. **Dual Routing Paradigms**:
   - **Declarative (`serve` / `Alloy.serve`)**: Bun/Alloy-compatible `routes` table with static routes, dynamic parameter paths (`/users/:id`), per-HTTP method route maps (`GET`, `POST`), wildcard routes (`/api/*`), static files, redirects, and fallback `fetch` handler.
   - **Object-Oriented (`App`)**: `rocket::build()` counterpart supporting `app.get()`, `app.post()`, `app.put()`, `app.delete()`, `app.patch()`, `app.head()`, `app.options()`, `app.launch()`, `app.stop()`.

2. **Sync & Async JavaScript Handlers**:
   - Route handlers support both synchronous return values (`() => response("pong")`) and `async` promises (`async () => response("pong")`) via automatic JS function promise-wrapping.

3. **Path Parameter Parsing & Routing**:
   - Supports both Express/Bun syntax (`:param`, `*`) and Rocket syntax (`<param>`, `<rest..>`), automatically extracting path parameters into `req.params`.

4. **Port Allocation & Environmental Overrides**:
   - Handles `port: 0` for random OS port allocation.
   - Reads `BUN_PORT`, `PORT`, `NODE_PORT` environment variables with default fallback to `3000`.

5. **Configuration Engine (`Config`)**:
   - Figment-backed Rocket server configuration with profile defaults (`Config.debugDefault()`, `Config.releaseDefault()`), `tryFrom()` override merging, and getters/setters for all Rocket parameters.

6. **TypeScript SDK Generator (`codegen`)**:
   - Converts Rocket routes into strongly typed TypeScript clients (`generateClient`), route type maps (`generateTypes`), and URI builders (`generateModule`).

---

## What Is Missing & Limitations

1. **Managed State & Request Guards**:
   - Rocket's `State<T>` and custom request guards (e.g., authentication guards, database connection pools) run in Rust. There is currently no NAPI wrapper to pass managed state or custom request guards from JS/TS into route handlers.

2. **Fairings**:
   - Rocket's lifecycle fairings (`on_ignite`, `on_liftoff`, `on_request`, `on_response`, `on_shutdown`) are currently internal-only (used for capturing shutdown handles). Exposing custom fairing hooks to JS functions is not yet implemented.

3. **Multipart & Streaming Data**:
   - Large request bodies are currently capped at 1MB and read fully into memory (`Vec<u8>`). Rocket's streaming `Data` pipeline and multipart form handling (`FromForm`) are not yet bridged for streaming directly to Node.js streams.

4. **Catchers / Error Boundaries**:
   - Custom HTTP error catchers (e.g. `#[catch(404)]`) are supported at the low-level `Rocket` class (`rocket.register()`), but are not yet surfaced in `serve()` / `App` options.

5. **WebSocket & Connection Upgrades**:
   - Upgrading HTTP connections to WebSockets or raw I/O streams is supported natively in Rocket 0.5.1 but not yet bridged to Node.js / Alloy socket interfaces.

---

## Planned Next Steps for Rocket NAPI Bindings

### Phase 1: Enhanced Request Context & Form Support

- [ ] **Streaming Body & Form Parsing**: Bridge Rocket's `Data` stream to Node.js `ReadableStream` / `Buffer` for large payloads and multipart file uploads (`FromForm`).
- [ ] **Catchers in `serve` / `App`**: Add `catchers` option to `serve({ routes, catchers: { 404: handler, 500: handler } })` and `app.catch(code, handler)`.

### Phase 2: State Management & Request Guards

- [ ] **State & Context Injections**: Allow JS callers to register global state (`app.manage(state)`) accessible in `req.state`.
- [ ] **Request Guard Middleware**: Allow JS functions to run pre-request guards (`app.guard(fn)`) that return early responses or inject context.

### Phase 3: Fairings & Lifecycle Hooks

- [ ] **JS Fairings**: Support `app.attach({ onResponse: fn, onRequest: fn })` to hook into Rocket request/response processing.

### Phase 4: WebSockets & Full-Stack Extensions

- [ ] **WebSocket Upgrades**: Expose Rocket HTTP upgrades to JS WebSocket handlers.
- [ ] **HTML / Asset Bundle Imports**: Integrate pre-bundled HTML asset serving for full-stack applications.

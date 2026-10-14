# Implementation Status & Roadmap

This document outlines the current implementation status of Rocket NAPI bindings for TypeScript (Node.js) in `@alloy-ts/server` (`alloy-server-native`) and plans the next steps for completing the NAPI surface.

---

## What is Implemented & Working

1. **Rocket Runtime & Tokio Integration (`src/server.rs`)**
   - Background multi-threaded Tokio runtime driving Rocket server execution cleanly without blocking Node's main event loop.
   - Instance-bound graceful shutdown (`server.stop()`) using `Arc<Mutex<Option<Shutdown>>>`.

2. **Bun / Alloy `serve` API (`Alloy.serve`)**
   - Support for option object configuration (`{ routes, fetch, port, hostname, unix }`).
   - Dynamic path parameter extractions (`/users/:id`).
   - Wildcard route matching (`/api/*`).
   - Per-HTTP method route maps (`{ GET: ..., POST: ... }`).
   - HTML string responses & import objects (`{ html: ... }` / `{ default: ... }`).
   - Static file serving (`file("./path")`).
   - Unmatched route catch-all `fetch` fallback handlers.
   - Environment variable port fallback cascade (`$BUN_PORT`, `$PORT`, `$NODE_PORT`, default `3000`).
   - Dynamic random port binding (`port: 0`) with post-liftoff URL and port resolution.

3. **Imperative Builder API (`App`)**
   - 1:1 counterpart of `rocket::build().mount("/", routes![...])`.
   - Route registration methods (`app.get`, `app.post`, `app.put`, `app.delete`, `app.patch`, `app.head`, `app.options`, `app.mount`, `app.launch`).

4. **Rocket Configuration (`Config`)**
   - NAPI class mirroring `rocket::Config` and `rocket::figment`.
   - Static default factories (`Config.debugDefault()`, `Config.releaseDefault()`, `Config.defaultConfig()`).
   - Partial override initialization (`Config.tryFrom(init)`, `Config.from(init)`).
   - Figment environment variable and `Rocket.toml` extraction (`Config.figment()`).
   - Getters & setters for `address`, `port`, `workers`, `maxBlocking`, `ident`, `ipHeader`, `limits`, `shutdown`, `logLevel`, `cliColors`.

5. **Rocket Core Struct NAPI Bindings**
   - `Route` (`src/route.rs`): Direct NAPI wrapper around `rocket::Route` (`new Route(method, uri, handler)`).
   - `RouteUri` (`src/route_uri.rs`): Direct NAPI wrapper around `rocket::route::RouteUri` (`new RouteUri(base, path)`).
   - `Handler` (`src/handler.rs`): NAPI wrapper around `rocket::route::Handler` supporting JS async route handlers (`new Handler(callback)`).
   - `Catcher` (`src/catcher.rs`): NAPI wrapper around `rocket::catcher::Catcher` supporting JS error handler callbacks for HTTP status codes (`new Catcher(code, handler)`).

6. **TypeScript Code Generation (`codegen.rs`)**
   - `parseRoute` / `parseRoutes` to parse Rocket & Bun style route patterns into structured parameter definitions.
   - `toRocketPath` path pattern translation.
   - `generateTypes`, `generateClient`, `generateModule` emitting strongly-typed TypeScript SDKs and route map interfaces.

---

## What is Missing & Future Roadmap

To achieve 100% feature parity with Rocket's Rust features across NAPI, the following features are planned for future iterations:

### Phase 1: Custom Catchers & Middleware / Fairings

- **TypeScript Middleware / Fairings**: Expose `app.attach(fairing)` with `onRequest`, `onResponse`, and `onShutdown` hooks in JS.

### Phase 2: Cookies & Managed State

- **Encrypted Cookie Jar API**: Expose `req.cookies.get(name)` and `req.cookies.setPrivate(name, value)` bridging Rocket's CookieJar.
- **Managed Application State**: NAPI binding to attach and extract state objects across requests (`app.manage(state)`).

### Phase 3: Streaming & WebSockets

- **Streaming Response & Request Bodies**: Support `ReadableStream` / async iterators for streaming large files or responses.
- **WebSocket Upgrade Handling**: Native HTTP upgrade handling for WebSocket connections (`req.upgrade()`).

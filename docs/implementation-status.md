# Rocket NAPI Bindings - Implementation Status & Roadmap

This document outlines the current implementation state of `@alloy-ts/server` (NAPI bindings for the Rocket web framework), identifying completed features, current gaps, and planned next steps.

---

## 1. Feature Status Matrix

| Component                            |     Status      | Details                                                                                                      |
| :----------------------------------- | :-------------: | :----------------------------------------------------------------------------------------------------------- |
| **Server Lifecycle (`Alloy.serve`)** | ✅ **Complete** | Starts Rocket on Tokio runtime; supports `url`, `port`, and graceful `stop()`.                               |
| **Static & Dynamic Routing**         | ✅ **Complete** | Supports static paths, `:param`, `<param>`, `<param..>` dynamic parameters, and wildcard `/api/*` routes.    |
| **Route Structure (`Route`)**        | ✅ **Complete** | NAPI class exposing `name`, `method`, `uri`, `rank`, `format`, computed default ranking, and custom ranking. |
| **Per-Method Handlers**              | ✅ **Complete** | Accepts `{ GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS }` route maps.                                       |
| **Response Helpers**                 | ✅ **Complete** | `Response.json()`, `Response.redirect()`, `Response` status and headers getters/setters.                     |
| **File & HTML Manifest Serving**     | ✅ **Complete** | `Alloy.file()` for lazy file reads; static object manifests for single-page app HTML imports.                |
| **Fallback Handler (`fetch`)**       | ✅ **Complete** | Ranked catch-all route (`/<_..>`) serving unmatched paths via TS callbacks.                                  |
| **Port Environment Overrides**       | ✅ **Complete** | Checks `BUN_PORT`, `PORT`, and `NODE_PORT` before defaulting to `3000`.                                      |
| **Rocket `App` Builder**             | ✅ **Complete** | 1:1 TS counterpart for `rocket::build()`, `#[get]`, `routes!`, `#[launch]`.                                  |
| **Configuration (`Config`)**         | ✅ **Complete** | Figment provider integration with `Limits`, `Shutdown`, `TlsConfig`, `MutualTls`.                            |
| **Codegen Helpers**                  | ✅ **Complete** | `generate_routes_manifest` and type-safe `build_uri` parameter interpolator.                                 |
| **Managed State (`State<T>`)**       |  ⏳ _Planned_   | Injecting shared server state into JS request handler contexts (`req.state`).                                |
| **Fairings & Middleware**            |  ⏳ _Planned_   | TS callbacks for Rocket `on_request` and `on_response` fairing hooks.                                        |
| **Multipart Form Parsing**           |  ⏳ _Planned_   | Streaming form/multipart body parsing into structured JS objects.                                            |
| **WebSockets / Upgrade**             |  ⏳ _Planned_   | Upgrading HTTP connections to WebSocket streams.                                                             |

---

## 2. Review of Current TS Integration

- **Thread Safety**: Request and response payloads pass between Tokio background worker threads and the Node.js event loop using NAPI `ThreadsafeFunction`.
- **Param Extraction**: Path parameters (`:id`, `<id>`) are parsed in Rust and serialized as plain JavaScript objects (`req.params.id`).
- **OXC / Node ESM Compatibility**: Verified compatibility when running examples with `node --import @oxc-node/core/register`.

---

## 3. Next Steps & Roadmap for Finishing NAPI Bindings

1. **Managed State (`rocket.manage`)**:
   - Allow JS code to register global state objects on launch and expose them on `RequestPayload` (`req.state`).
2. **Fairing Middleware**:
   - Provide NAPI bindings for Rocket Fairings so TS developers can attach global request/response logging, header modification, and authentication middleware.
3. **Multipart & Form Parsing**:
   - Integrate Rocket's `Form` guard to parse incoming `application/x-www-form-urlencoded` and `multipart/form-data` requests into `req.form`.
4. **Custom Catchers**:
   - Allow registering custom status error handlers (e.g. 400, 404, 500) via `app.register_catcher(code, handler)`.

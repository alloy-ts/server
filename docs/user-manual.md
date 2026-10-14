# @alloy-ts/server User Manual

This manual provides a comprehensive guide to using `@alloy-ts/server` — Node.js and TypeScript NAPI bindings for the Rust [Rocket](https://rocket.rs/) web framework and code generation utilities.

---

## Table of Contents

1. [Overview & Architecture](#overview--architecture)
2. [Installation & Setup](#installation--setup)
3. [High-Performance HTTP Server (`serve`)](#high-performance-http-server-serve)
   - [Basic Setup](#basic-setup)
   - [Route Map Syntax](#route-map-syntax)
   - [Response Helpers](#response-helpers)
   - [Fallback Handlers](#fallback-handlers)
   - [Server Handle & Shutdown](#server-handle--shutdown)
4. [Rocket Server Lifecycle (`Rocket`)](#rocket-server-lifecycle-rocket)
   - [Creating a Rocket Instance](#creating-a-rocket-instance)
   - [Mounting Routes & Catchers](#mounting-routes--catchers)
   - [Igniting & Launching](#igniting--launching)
   - [Graceful Shutdown](#graceful-shutdown)
5. [Rocket Configuration (`Config`)](#rocket-configuration-config)
   - [Constructors & Factory Methods](#constructors--factory-methods)
   - [Configuration Parameters](#configuration-parameters)
   - [Limits & Shutdown Options](#limits--shutdown-options)
6. [TypeScript Code Generation (`codegen`)](#typescript-code-generation-codegen)
   - [Route Pattern Parsing](#route-pattern-parsing)
   - [Pattern Translation](#pattern-translation)
   - [Generating Route Map Types (`generateTypes`)](#generating-route-map-types-generatetypes)
   - [Generating Typed Fetch Clients (`generateClient`)](#generating-typed-fetch-clients-generateclient)
   - [Generating Full Modules (`generateModule`)](#generating-full-modules-generatemodule)

---

## Overview & Architecture

`@alloy-ts/server` bridges the Rocket web framework into Node.js and TypeScript via NAPI-RS (`alloy-server-native`).

Key design highlights:

- **Rust Performance**: Rocket runs on a dedicated, multi-threaded Tokio runtime managed in native Rust code, handling HTTP request processing and routing at native speed.
- **Node.js/Bun Compatibility**: Provides Bun/Alloy-style `serve()` APIs (`new Response()`, `Response.json()`, `file()`) while leveraging Rocket under the hood.
- **Complete Codegen Pipeline**: Translates route specifications into strongly-typed TypeScript clients, route interface maps, and Rocket-compatible `uri()` builders.

---

## Installation & Setup

Install `@alloy-ts/server` into your Node.js project:

```bash
npm install @alloy-ts/server
```

Requirements:

- Node.js >= 18
- Modern 64-bit OS (Linux, macOS, Windows)

---

## High-Performance HTTP Server (`serve`)

### Basic Setup

The `serve()` function offers a Bun/Alloy-style routing API backed by Rocket:

```ts
import { file, response, responseJson, responseRedirect, serve } from "@alloy-ts/server";

const server = serve(
  {
    // Static text response
    "/api/status": response("OK"),

    // Dynamic path route with path parameter extraction
    "/users/:id": (req) => response(`Hello User ${req.params.id}!`),

    // Per-HTTP method routing table
    "/api/posts": {
      GET: () => response("List posts"),
      POST: async (req) => responseJson({ created: true }),
    },

    // Wildcard match for unmatched /api/* subpaths
    "/api/*": responseJson({ message: "Not found" }, { status: 404 }),

    // Redirect response
    "/blog/hello": responseRedirect("/blog/hello/world"),

    // Lazily served static file
    "/favicon.ico": file("./favicon.ico"),
  },
  // Fallback for unmatched routes
  (req) => response("Not Found", { status: 404 }),
  {
    port: 3000,
    hostname: "0.0.0.0",
  },
);

console.log(`Server running at ${server.url}`);
```

### Route Map Syntax

`serve()` accepts both Bun/Alloy route syntax and Rocket route syntax:

| Pattern                           | Meaning                    | Example                                 |
| --------------------------------- | -------------------------- | --------------------------------------- |
| `/path/:param` or `/path/<param>` | Dynamic path parameter     | `/users/:id` extracts `req.params.id`   |
| `/path/*` or `/path/<_..>`        | Multi-segment wildcard     | `/api/*` matches any `/api/...` subpath |
| `?a&b` or `?<a>&<b>`              | Query parameter extraction | `req.query` contains raw query string   |

### Response Helpers

- `response(body?, init?)`: Plain text / custom response object.
- `responseJson(data, init?)`: Returns JSON response with `Content-Type: application/json`.
- `responseRedirect(url, status?)`: Returns redirect response (defaults to status 302).
- `file(path)`: Marks a file to be served from disk.

### Server Handle & Shutdown

`serve()` synchronously returns a `Server` handle:

```ts
console.log(server.url); // "http://0.0.0.0:3000"
console.log(server.port); // 3000

// Gracefully stop server
server.stop();
```

---

## Rocket Server Lifecycle (`Rocket`)

When fine-grained, programmatic control over Rocket is needed, use the `Rocket` class:

### Creating a Rocket Instance

```ts
import { Rocket } from "@alloy-ts/server";

const rocket = new Rocket({
  address: "127.0.0.1",
  port: 8000,
  workers: 4,
  logLevel: "normal",
});
```

### Mounting Routes & Catchers

```ts
// Mount a route
rocket.mount("/", "GET", "/users/<id>", async (req) => ({
  status: 200,
  headers: [{ name: "Content-Type", value: "text/plain" }],
  body: Buffer.from(`User ID: ${req.params?.id}`),
}));

// Register an error catcher for status 404
rocket.register("/", 404, async (req) => ({
  status: 404,
  headers: [{ name: "Content-Type", value: "application/json" }],
  body: Buffer.from(JSON.stringify({ error: "Custom 404 Not Found" })),
}));
```

### Igniting & Launching

```ts
// Launch the server; resolves when server shuts down
await rocket.launch();
```

### Graceful Shutdown

```ts
rocket.shutdown();
```

---

## Rocket Configuration (`Config`)

The `Config` class is a NAPI mirror of `rocket::config::Config`.

### Constructors & Factory Methods

```ts
import { Config } from "@alloy-ts/server";

// Seeded with Rocket's debug defaults
const debugConfig = Config.debugDefault();

// Seeded with Rocket's release defaults
const releaseConfig = Config.releaseDefault();

// Loaded from Rocket.toml and ROCKET_* environment variables
const figmentConfig = Config.figment();

// Created with partial overrides
const customConfig = Config.tryFrom({
  port: 8080,
  address: "0.0.0.0",
  logLevel: "debug",
  shutdown: { grace: 5, mercy: 10 },
});
```

### Configuration Parameters

Available getters and setters on `Config`:

- `profile`: Profile name (`"debug"`, `"release"`, custom).
- `address`: IP address to listen on (`"127.0.0.1"`, `"0.0.0.0"`).
- `port`: Port number (e.g. `8000`).
- `workers`: Worker thread count.
- `maxBlocking`: Limit on threads for blocking tasks.
- `keepAlive`: Keep-alive timeout in seconds.
- `ident`: Server identity header string (or `null`/`undefined`).
- `ipHeader`: Header to inspect for client IP (e.g. `"X-Real-IP"`).
- `limits`: Read size limits map (`{ form: "64KiB", json: "10MiB" }`).
- `tempDir`: Temporary files directory path.
- `shutdown`: Shutdown configuration object.
- `logLevel`: Logging level (`"off"`, `"critical"`, `"normal"`, `"debug"`).
- `cliColors`: Boolean toggle for colored CLI logs.

---

## TypeScript Code Generation (`codegen`)

The codegen module provides runtime code generation utilities to convert route patterns into TypeScript types, client SDKs, and route tables.

### Route Pattern Parsing

```ts
import { parseRoute, parseRoutes } from "@alloy-ts/server";

const route = parseRoute({
  method: "GET",
  path: "/users/:id/posts/*?<q>",
  response: "Post[]",
});

console.log(route.key); // "GET /users/:id/posts/*"
console.log(route.rocketPath); // "/users/<id>/posts/<_..>"
console.log(route.name); // "getUsersByIdPosts"
console.log(route.params); // Dynamic path parameters metadata
console.log(route.query); // Query parameters metadata
```

### Pattern Translation

```ts
import { toRocketPath } from "@alloy-ts/server";

const rocketSyntax = toRocketPath("/users/:id/*");
// "/users/<id>/<_..>"
```

### Generating Route Map Types (`generateTypes`)

Emit TypeScript route map interface definitions:

```ts
import { parseRoutes, generateTypes } from "@alloy-ts/server";

const routes = parseRoutes([
  { method: "GET", path: "/users/:id", response: "User" },
  { method: "POST", path: "/users", body: "CreateUser", response: "User" },
]);

const typesCode = generateTypes(routes, {
  typesName: "ApiRoutes",
});
```

### Generating Typed Fetch Clients (`generateClient`)

Emit a typed `fetch` client class with a `uri()` helper:

```ts
import { parseRoutes, generateClient } from "@alloy-ts/server";

const routes = parseRoutes([
  { method: "GET", path: "/users/:id", name: "getUser", response: "User" },
]);

const clientCode = generateClient(routes, {
  clientName: "UserApiClient",
  baseUrl: "https://api.example.com",
});
```

### Generating Full Modules (`generateModule`)

Emit a complete module containing `ROUTES` constants, route interfaces, `uri()` route builder, and client SDK:

```ts
import { parseRoutes, generateModule } from "@alloy-ts/server";

const routes = parseRoutes([
  { method: "GET", path: "/users/:id", response: "User" },
  { method: "POST", path: "/users", body: "CreateUser", response: "User" },
]);

const fullModule = generateModule(routes, {
  clientName: "ApiClient",
  baseUrl: "http://localhost:3000",
});
```

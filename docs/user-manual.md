# `@alloy-ts/server` User Manual

`@alloy-ts/server` provides high-performance TypeScript / Node.js NAPI bindings for the **Rocket** web framework (Rust). It bridges Rocket's async multi-threaded runtime into Node.js while exposing Bun.serve and Express/Rocket-style server APIs, configuration management, and TypeScript SDK codegen utilities.

---

## Table of Contents

1. [Architecture & Overview](#architecture--overview)
2. [Quick Start](#quick-start)
3. [Server APIs](#server-apis)
   - [Bun/Alloy Compatibility (`Alloy.serve` / `Bun.serve`)](#bunalloy-compatibility-alloyserve--bunserve)
   - [Fluent Application Builder (`App`)](#fluent-application-builder-app)
4. [Request & Response Handling](#request--response-handling)
   - [Request Object Helpers](#request-object-helpers)
   - [Response Helpers & Factory Methods](#response-helpers--factory-methods)
   - [Static File Serving](#static-file-serving)
5. [Route Metadata & Default Ranking (`Route`)](#route-metadata--default-ranking-route)
6. [Custom Handlers & Outcomes (`Handler` & `Outcome`)](#custom-handlers--outcomes-handler--outcome)
7. [Rocket Configuration (`Config`)](#rocket-configuration-config)
8. [TypeScript Code Generation (`codegen`)](#typescript-code-generation-codegen)

---

## Architecture & Overview

When launching a server via `@alloy-ts/server`:
- Rocket runs on a dedicated multi-threaded Tokio runtime in a background OS thread.
- Node's main event loop remains unblocked.
- Every incoming HTTP request is converted into a native JS request object containing `method`, `path`, `query`, `headers`, `params`, `req.json()`, and `req.text()`.
- JS handlers resolve `Promise<Response>` or `Outcome` payloads back to Rocket worker threads.

---

## Quick Start

```ts
import { Alloy, response } from "@alloy-ts/server";

const server = Alloy.serve({
  port: 3000,
  routes: {
    "/": () => response("Hello from Rocket NAPI!"),
    "/users/:id": (req) => response(`User ID: ${req.params.id}`),
  },
});

console.log(`Server running at ${server.url}`);
```

---

## Server APIs

### Bun/Alloy Compatibility (`Alloy.serve` / `Bun.serve`)

Start an HTTP server using a single configuration object:

```ts
import { Alloy, Bun, response, responseJson, responseRedirect } from "@alloy-ts/server";

const server = Alloy.serve({
  port: 8080, // Defaults to $BUN_PORT, $PORT, $NODE_PORT or 3000
  hostname: "0.0.0.0",
  routes: {
    // Static response
    "/api/status": response("OK"),

    // Dynamic path segment matching
    "/users/:id": (req) => response(`User: ${req.params.id}`),

    // Per-HTTP method map
    "/api/posts": {
      GET: () => response("List posts"),
      POST: async (req) => {
        const body = await req.json();
        return responseJson({ created: true, ...body });
      },
    },

    // Wildcard matching
    "/api/*": responseJson({ error: "Not Found" }, { status: 404 }),

    // Redirects
    "/old-path": responseRedirect("/new-path"),

    // Lazy static file loading
    "/favicon.ico": Alloy.file("./favicon.ico"),
  },

  // Fallback handler for unmatched routes
  fetch(req) {
    return response("Fallback 404", { status: 404 });
  },
});

// Stop server gracefully
server.stop();
```

#### Random Port Allocation & Sockets

```ts
// Set port: 0 for random available port allocation by OS
const server = Alloy.serve({ port: 0, fetch: () => response("OK") });
console.log(server.port); // Allocated port number

// Unix domain socket
Alloy.serve({ unix: "/tmp/my-socket.sock", fetch: () => response("OK") });

// Linux abstract namespace socket
Alloy.serve({ unix: "\0my-abstract-socket", fetch: () => response("OK") });
```

### Fluent Application Builder (`App`)

The `App` class translates canonical Rocket quickstart idioms (`rocket::build()`, `#[get("/")]`, `routes!`, `#[launch]`) into TypeScript:

```ts
import { App, response } from "@alloy-ts/server";

const app = new App();

app.get("/", async () => response("Hello, world!"));
app.get("/users/:id", async (req) => response(`User: ${req.params.id}`));
app.post("/items", async (req) => {
  const body = await req.json();
  return responseJson({ created: true, ...body });
});

const server = app.launch({ port: 3000 });
console.log(`Server running at ${server.url}`);
```

---

## Request & Response Handling

### Request Object Helpers

Every JS handler receives a `req` object containing:
- `req.method`: Upper-cased HTTP method string.
- `req.path`: Normalized request path.
- `req.query`: Raw query string.
- `req.headers`: Map of HTTP headers.
- `req.params`: Key-value map of dynamic path segments (`req.params.id`).
- `req.body`: `Uint8Array` / raw body bytes.
- `await req.json()`: Parses request body as JSON.
- `await req.text()`: Returns request body as UTF-8 string.

### Response Helpers & Factory Methods

- `response(body?, init?)`: Creates a plain response object.
- `responseJson(data, init?)`: Creates JSON response with `Content-Type: application/json`.
- `responseRedirect(url, status?)`: Creates redirect response (`Location` header set, default status 302).

### Static File Serving

Use `file(path)` / `Alloy.file(path)` / `Bun.file(path)` to serve static files lazily directly from Rocket:

```ts
import { file } from "@alloy-ts/server";

routes: {
  "/logo.png": file("./public/logo.png"),
}
```

---

## Route Metadata & Default Ranking (`Route`)

Rocket computes route ranks based on static vs dynamic path and query components. The `Route` class exposes Rocket's default ranking logic to TypeScript:

```ts
import { Route } from "@alloy-ts/server";

const route = new Route("GET", "/users/:id");
console.log(route.rank); // -5 (partial path, no query)

const staticRoute = new Route("GET", "/?foo");
console.log(staticRoute.rank); // -12 (static path, static query)

// Explicit ranking
const customRoute = Route.ranked(1, "POST", "/foo?bar", "handlerName", "json");
expect(customRoute.rank).toBe(1);

// Map base URI
const mapped = customRoute.mapBase((base) => `/api/v1${base}`);
```

---

## Custom Handlers & Outcomes (`Handler` & `Outcome`)

Build custom Rocket request handlers with explicit outcomes:

```ts
import { Handler, outcomeError, outcomeForward, outcomeFrom, response } from "@alloy-ts/server";

const customHandler = new Handler(async (req) => {
  if (req.headers["authorization"]) {
    return outcomeFrom(response("Authorized"));
  } else if (req.path.startsWith("/skip")) {
    return outcomeForward(404);
  } else {
    return outcomeError(401);
  }
});
```

---

## Rocket Configuration (`Config`)

`Config` mirrors Rocket's configuration provider system (defaults, `Rocket.toml`, and `ROCKET_*` environment variables):

```ts
import { Config } from "@alloy-ts/server";

// Debug / Release defaults
const debugCfg = Config.debugDefault();
const releaseCfg = Config.releaseDefault();

// Partial overrides
const cfg = Config.tryFrom({
  port: 8000,
  address: "0.0.0.0",
  logLevel: "debug",
  keepAlive: 10,
  shutdown: { grace: 5, mercy: 5, ctrlc: true },
});

// Mutate properties
cfg.port = 8080;
cfg.logLevel = "normal";
```

---

## TypeScript Code Generation (`codegen`)

Generate strongly-typed client SDKs, route maps, and URI builders from route specifications:

```ts
import { parseRoutes, generateModule, toRocketPath } from "@alloy-ts/server";

// Express/Bun -> Rocket route translation
const rocketPath = toRocketPath("/users/:id/*"); // "/users/<id>/<_..>"

// Generate TS SDK module
const routes = parseRoutes([
  { method: "GET", path: "/users/:id", response: "User" },
  { method: "POST", path: "/users", body: "CreateUser", response: "User" },
]);

const tsCode = generateModule(routes, { baseUrl: "http://localhost:3000" });
```

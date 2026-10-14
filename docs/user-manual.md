# Alloy Rocket Server — User Manual

Node.js (TypeScript) NAPI bindings for the [Rocket web framework](https://rocket.rs/). High-performance HTTP routing, server lifecycle control, Figment configuration management, custom `Handler` trait abstractions, and automated TypeScript client/type code generation.

---

## Getting Started

### Installation & Prerequisites

Build native NAPI bindings locally using Node.js and Rust:

```bash
npm install
npm run build:napi
```

Import `@alloy-ts/server` in your TypeScript or JavaScript application:

```ts
import { Alloy, response, responseJson } from "@alloy-ts/server";
```

---

## HTTP Server API (`Alloy.serve` & `App`)

`@alloy-ts/server` provides two ways to define and run your web server:

1. **`Alloy.serve(options)`**: Bun/Alloy-style declarative options object.
2. **`App`**: Rocket-style fluent builder API.

### 1. Declarative Options (`Alloy.serve`)

```ts
import { Alloy, file, response, responseJson, responseRedirect } from "@alloy-ts/server";

const server = Alloy.serve({
  port: Number(process.env["PORT"] ?? 3000),
  hostname: "127.0.0.1",
  routes: {
    // Static response
    "/api/status": response("OK"),

    // Dynamic path parameter
    "/users/:id": (req) => {
      return response(`Hello User ${req.params["id"]}!`);
    },

    // Per-HTTP method handlers
    "/api/posts": {
      GET: () => responseJson(["post1", "post2"]),
      POST: async (req) => {
        const bodyText = req.body ? new TextDecoder().decode(Uint8Array.from(req.body)) : "{}";
        const body = JSON.parse(bodyText);
        return responseJson({ created: true, ...body });
      },
    },

    // Wildcard fallback route
    "/api/*": responseJson({ error: "API route not found" }, { status: 404 }),

    // Redirect
    "/blog/hello": responseRedirect("/blog/hello/world"),

    // Lazy file serving
    "/favicon.ico": file("./favicon.ico"),
  },

  // Global fallback for unmatched routes
  fetch(req) {
    return response(`Not Found: ${req.path}`, { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);
```

### 2. Fluent Builder (`App`)

For developers accustomed to Rocket's `rocket::build().mount("/", routes![...])` pattern:

```ts
import { App, response, responseJson } from "@alloy-ts/server";

const app = new App({ port: 8080, hostname: "127.0.0.1" });

app
  .get("/", () => response("Hello from Rocket!"))
  .get("/users/:id", (req) => response(`User ${req.params["id"]}`))
  .post("/api/data", async (req) => responseJson({ received: true }));

const server = app.launch();
console.log(`App running at ${server.url}`);
```

---

## Handler Bindings (`Handler` & Outcomes)

For custom library-based or plugin mechanism route handlers (mirroring Rocket's `Handler` trait), `@alloy-ts/server` exposes `Handler` and outcome constructors:

```ts
import {
  Handler,
  HandlerOutcomeKind,
  createHandlerOutcomeSuccess,
  createHandlerOutcomeError,
  createHandlerOutcomeForward,
} from "@alloy-ts/server";

// Create custom handler wrapper
const customHandler = new Handler("CustomKindHandler", async (req) => {
  if (req.uri === "/special") {
    return createHandlerOutcomeSuccess(200, "Special content", [["X-Custom", "true"]]);
  } else if (req.uri === "/forbidden") {
    return createHandlerOutcomeError(403);
  } else {
    return createHandlerOutcomeForward(null);
  }
});

console.log(`Handler initialized: ${customHandler.name}`);
```

---

## Route Syntax & Request/Response Helpers

### Route Pattern Syntax

| Pattern             | Description                 | Rocket Equivalent | Example Match                                  |
| ------------------- | --------------------------- | ----------------- | ---------------------------------------------- |
| `:name` / `<name>`  | Single dynamic path segment | `<name>`          | `/users/42` -> `req.params.id = "42"`          |
| `*` / `<name..>`    | Multi-segment glob          | `<_..>`           | `/api/docs/setup` -> `req.params._`            |
| `?a&b` / `?<a>&<b>` | Query parameters            | `?<a>&<b>`        | `/search?q=rocket` -> `req.query = "q=rocket"` |

### Request Input (`RequestInput`)

Every route handler callback receives a `RequestInput` object:

```ts
export interface RequestInput {
  method: string;
  path: string;
  query: string;
  headers: Record<string, string>;
  body?: Array<number>; // raw body bytes
  params: Record<string, string>;
}
```

### Response Helpers

- **`response(body?, init?)`**: Creates a text/plain response.
- **`responseJson(data, init?)`**: Stringifies data to JSON and sets `Content-Type: application/json`.
- **`responseRedirect(url, status?)`**: Constructs a redirect response (default status `302`).
- **`file(path)`**: Constructs a lazily-served file marker.

---

## Configuration (`Config`)

Rocket's Figment-backed configuration is exposed via the NAPI `Config` class:

```ts
import { Config } from "@alloy-ts/server";

// Seeded with Rocket's debug defaults
const config = new Config();

// Mutate properties
config.port = 8080;
config.address = "127.0.0.1";
config.workers = 8;
config.maxBlocking = 256;
config.ident = "MyCustomServer";
config.logLevel = "normal";

// Set streaming limits
config.limits = {
  limits: {
    json: "10MiB",
    form: "64KiB",
  },
};

// Set shutdown parameters
config.shutdown = {
  ctrlc: true,
  grace: 3,
  mercy: 5,
  force: true,
  signals: ["term", "hup"],
};

// Merge overrides from plain objects
const customConfig = Config.tryFrom({
  port: 9000,
  logLevel: "debug",
});
```

---

## TypeScript Code Generation (`codegen`)

Generate strongly-typed fetch SDKs and TypeScript type declarations directly from your route table:

```ts
import { generateClient, generateModule, parseRoutes } from "@alloy-ts/server";

const routes = parseRoutes([
  { method: "GET", path: "/users/:id", response: "User" },
  { method: "POST", path: "/users", body: "CreateUser", response: "User" },
]);

// Emit typed client SDK with uri() builder
const clientCode = generateClient(routes, {
  clientName: "UserApiClient",
  baseUrl: "http://127.0.0.1:3000",
});

// Emit full module (ROUTES constant + types + client)
const fullModule = generateModule(routes);
```

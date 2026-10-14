# `@alloy-ts/server` User Manual

High-performance Node.js / TypeScript NAPI bindings for the **Rocket** web framework.

---

## Overview

`@alloy-ts/server` connects Rocket's asynchronous Rust runtime to Node.js and TypeScript via NAPI. It provides two ergonomic server APIs:

1. **`Alloy.serve` / `serve`**: Bun/Alloy-style declarative server setup (`routes`, `fetch` fallback, `port`, `hostname`).
2. **`App`**: Object-oriented application builder mirroring `rocket::build()` and `#[launch]` (`app.get()`, `app.post()`, `app.launch()`).

It also includes Rocket server configuration (`Config`) and TypeScript SDK code generation utilities (`codegen`).

---

## Installation

```bash
npm install @alloy-ts/server
```

When running TypeScript files directly with Node.js, you can use `@oxc-node/core`:

```bash
node --import @oxc-node/core/register examples/server.ts
```

---

## Quick Start

### Declarative API (`serve` / `Alloy.serve`)

```ts
import { serve, response, responseJson, responseRedirect, file } from "@alloy-ts/server";

const server = serve(
  {
    // Static text route
    "/": () => response("Welcome to Rocket Server!"),

    // JSON API response
    "/api/status": () => responseJson({ status: "ok" }),

    // Dynamic path parameters (:id)
    "/users/:id": (req) => response(`User ID: ${req.params?.["id"]}`),

    // Per-HTTP method map
    "/api/posts": {
      GET: () => responseJson([{ id: 1, title: "First Post" }]),
      POST: async (req) => responseJson({ created: true, body: req.body }),
    },

    // Redirect
    "/docs": () => responseRedirect("/"),

    // Serve a static file from disk
    "/favicon.ico": file("./favicon.ico"),
  },

  // Fallback handler for unmatched routes
  (req) => responseJson({ error: "Not Found", path: req.path }, { status: 404 }),

  {
    port: 3000,
    hostname: "127.0.0.1",
  },
);

console.log(`Server running at ${server.url}`);
```

---

### Object-Oriented API (`App`)

The `App` class is the TypeScript equivalent of Rocket's `rocket::build()` builder:

```ts
import { App, response, responseJson } from "@alloy-ts/server";

const app = new App();

app.get("/", () => response("Hello, world!"));

app.get("/users/:id", (req) => response(`Hello User ${req.params?.["id"]}`));

app.post("/api/data", async (req) => responseJson({ received: req.body }));

// Starts listening on background Tokio runtime and returns Server handle
const server = app.launch({ port: 3000, hostname: "127.0.0.1" });

console.log(`Server listening at ${server.url}`);

// Stop gracefully when needed
// app.stop();
```

---

## Response Helpers

| Helper                           | Description                                      | Example                           |
| -------------------------------- | ------------------------------------------------ | --------------------------------- |
| `response(body?, init?)`         | Plain text or raw byte response                  | `response("OK", { status: 200 })` |
| `responseJson(data, init?)`      | JSON response (`Content-Type: application/json`) | `responseJson({ ok: true })`      |
| `responseRedirect(url, status?)` | Redirect response (default HTTP 302)             | `responseRedirect("/login")`      |
| `file(path)`                     | Lazy static file serving marker                  | `file("./assets/index.html")`     |

---

## Configuration (`Config`)

The `Config` class exposes Rocket's Figment-backed configuration engine:

```ts
import { Config } from "@alloy-ts/server";

// Seed with Rocket's debug or release defaults
const config = Config.debugDefault();

config.port = 8080;
config.address = "0.0.0.0";
config.workers = 16;
config.logLevel = "normal";

// Apply partial overrides
const customConfig = Config.tryFrom({
  port: 9000,
  limits: {
    limits: { json: "10MiB", form: "64KiB" },
  },
  shutdown: {
    grace: 10,
    mercy: 5,
  },
});
```

### Port Resolution Hierarchy

When starting a server with `serve` or `app.launch()`:

1. `options.port` if provided (`port: 0` allocates a random available port).
2. Environment variables: `BUN_PORT`, `PORT`, `NODE_PORT`.
3. Default port: `3000`.

---

## Code Generation (`codegen`)

Parse Rocket route patterns and generate strongly-typed TypeScript SDK clients:

```ts
import { parseRoutes, generateModule } from "@alloy-ts/server";

const routes = parseRoutes([
  { method: "GET", path: "/users/:id", response: "User" },
  { method: "POST", path: "/users", body: "CreateUser", response: "User" },
]);

const code = generateModule(routes, {
  clientName: "UserApiClient",
  baseUrl: "http://localhost:3000",
});

console.log(code);
```

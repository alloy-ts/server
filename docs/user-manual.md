# @alloy-ts/server User Manual

This manual provides a detailed guide to using `@alloy-ts/server`, a high-performance Node.js / TypeScript binding for the Rust **Rocket v0.5** web framework.

---

## Table of Contents

1. [Overview](#1-overview)
2. [Installation](#2-installation)
3. [Alloy HTTP Server (`Alloy.serve`)](#3-alloy-http-server-alloyserve)
4. [Application Builder (`App`)](#4-application-builder-app)
5. [Rocket Server Lifecycle (`Rocket`)](#5-rocket-server-lifecycle-rocket)
6. [Server Configuration (`Config`)](#6-server-configuration-config)
7. [TypeScript Code Generation (`codegen`)](#7-typescript-code-generation-codegen)
8. [Running Examples & TypeScript Execution](#8-running-examples--typescript-execution)

---

## 1. Overview

`@alloy-ts/server` bridges Rust's Rocket web framework to Node.js using native NAPI bindings. It provides:

- **Bun/Alloy-compatible `serve` API**: Fast, declarative routing with path parameters (`/users/:id`), method maps, wildcards (`/api/*`), file serving, and fallback handlers.
- **Rocket `App` Builder**: Method-chaining server builder (`app.get()`, `app.post()`, `app.mount()`) re-expressing Rocket's macros in TypeScript.
- **Direct Rocket Lifecycle (`Rocket`)**: Programmatic control over mounting, error catchers, ignition, and graceful shutdown.
- **Rocket Server Configuration (`Config`)**: Full mirror of Rocket's configuration options (`Config.debugDefault()`, `Config.tryFrom()`, limits, shutdown signals).
- **TypeScript Codegen**: Extraction of route parameters, Rocket/Alloy path conversion, typed fetch SDK generation, and handler boilerplate generation.

---

## 2. Installation

Install the package into your TypeScript or Node.js project:

```bash
npm install @alloy-ts/server
```

To run TypeScript files directly during development without a separate compilation step:

```bash
npm install -D @oxc-node/core
```

---

## 3. Alloy HTTP Server (`Alloy.serve`)

`Alloy.serve` (or `serve`) starts a Rocket web server behind a Bun-compatible interface.

### Basic Example

```ts
import { Alloy, Response } from "@alloy-ts/server";

const server = Alloy.serve({
  port: 3000,
  routes: {
    // Static response
    "/api/status": new Response("OK"),

    // Dynamic path parameter
    "/users/:id": (req) => new Response(`Hello User ${req.params.id}!`),

    // Per-HTTP method handlers
    "/api/posts": {
      GET: () => new Response("List posts"),
      POST: async (req) => Response.json({ created: true }),
    },

    // Wildcard route
    "/api/*": Response.json({ message: "Not found" }, { status: 404 }),

    // Redirect
    "/blog/hello": Response.redirect("/blog/hello/world"),

    // File serving
    "/favicon.ico": Alloy.file("./favicon.ico"),
  },

  // Fallback handler for unmatched routes
  fetch(req) {
    return new Response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);
```

### Configuration Options for `serve`

`Alloy.serve` accepts configuration options:

```ts
Alloy.serve({
  port: 8080, // Defaults to $BUN_PORT, $PORT, $NODE_PORT, or 3000
  hostname: "127.0.0.1", // Defaults to "0.0.0.0"
  unix: "/tmp/my-socket.sock", // Optional Unix domain socket path
  routes: {/* ... */},
  fetch(req) {
    /* ... */
  },
});
```

---

## 4. Application Builder (`App`)

The `App` class provides an Express/Koa-style builder API corresponding 1:1 to Rocket's `rocket::build()`, `#[get]`, `routes!`, and `#[launch]`.

### Quickstart Example (`examples/app-routes.ts`)

```ts
import { App, response } from "@alloy-ts/server";

const app = new App();

// Define route handlers
app.get("/", async () => response("Hello, world!"));
app.get("/users/:id", async (req) => response(`User: ${req.params.id}`));
app.post("/users", async () => response("User Created", { status: 201 }));

// Mount sub-routes under a prefix
app.mount("/api", {
  "/status": response("OK"),
});

// Launch server on background runtime
const server = app.launch({ port: 3000 });

console.log(`Server running at ${server.url}`);
```

---

## 5. Rocket Server Lifecycle (`Rocket`)

For low-level programmatic control, use the `Rocket` class:

```ts
import { Rocket } from "@alloy-ts/server";

const rocket = new Rocket({
  port: 8080,
  logLevel: "normal",
});

// Mount a route
rocket.mount("/", "GET", "/hello/<name>", (req) => {
  return {
    status: 200,
    headers: [{ name: "Content-Type", value: "text/plain" }],
    body: Buffer.from(`Hello ${req.params?.name}!`),
  };
});

// Register a 404 error catcher
rocket.register("/", 404, () => {
  return {
    status: 404,
    body: Buffer.from("Custom 404 Error"),
  };
});

// Launch server
const launchPromise = rocket.launch();

// Trigger graceful shutdown
setTimeout(() => {
  rocket.shutdown();
}, 5000);

await launchPromise;
```

---

## 6. Server Configuration (`Config`)

Rocket's configuration system (`Config`) allows configuring port, address, workers, limits, shutdown signals, and log levels.

```ts
import { Config } from "@alloy-ts/server";

// 1. Debug defaults
const debugConfig = Config.debugDefault();
console.log(debugConfig.port); // 8000
console.log(debugConfig.address); // 127.0.0.1

// 2. Release defaults
const releaseConfig = Config.releaseDefault();
console.log(releaseConfig.logLevel); // "critical"

// 3. Partial overrides with Config.tryFrom
const customConfig = Config.tryFrom({
  port: 4000,
  address: "0.0.0.0",
  workers: 8,
  maxBlocking: 256,
  logLevel: "normal",
  shutdown: {
    ctrlc: true,
    grace: 5,
    mercy: 5,
  },
  limits: {
    limits: {
      json: "2MiB",
      form: "64KiB",
    },
  },
});
```

---

## 7. TypeScript Code Generation (`codegen`)

The `codegen` module provides runtime utilities for route pattern parsing, path syntax conversion, and TypeScript code generation.

### Route Pattern Parsing & Path Translation

```ts
import { parseRocketRoutePattern, translatePatternToAlloy, toRocketPath } from "@alloy-ts/server";

// Extract path & query parameters
const params = parseRocketRoutePattern("/users/<id>?<age>&<name..>");

// Convert Rocket path syntax to Alloy syntax
const alloy = translatePatternToAlloy("/users/<id>/<rest..>");
// Output: "/users/:id/*"

// Convert Alloy path syntax to Rocket syntax
const rocket = toRocketPath("/users/:id/*");
// Output: "/users/<id>/<_..>"
```

### Fetch Client SDK Generation

```ts
import { generateTypescriptClient } from "@alloy-ts/server";

const clientSdk = generateTypescriptClient(
  [
    {
      method: "GET",
      path: "/users/<id>",
      name: "getUser",
      returnType: "Promise<{ id: string; name: string }>",
    },
    {
      method: "POST",
      path: "/users",
      name: "createUser",
      bodyType: "{ name: string }",
      returnType: "Promise<{ id: string; name: string }>",
    },
  ],
  { clientName: "UserApiClient", baseUrl: "http://localhost:3000" },
);

console.log(clientSdk.code);
```

### Handler Boilerplate & Interface Generation

```ts
import { generateRouteHandler, generateTypescriptInterface } from "@alloy-ts/server";

// Route handler
const handler = generateRouteHandler({ method: "GET", path: "/users/:id" });

// TypeScript Interface
const userInterface = generateTypescriptInterface("User", [
  { name: "id", tsType: "string", optional: false, description: "Unique ID" },
  { name: "name", tsType: "string", optional: false },
]);
```

---

## 8. Running Examples & TypeScript Execution

You can run any example in `examples/` directly using Node.js with `@oxc-node/core`:

```bash
# Run Bun/Alloy server example
node --import @oxc-node/core/register examples/server.ts

# Run Rocket App quickstart example
node --import @oxc-node/core/register examples/app-routes.ts

# Run Codegen example
node --import @oxc-node/core/register examples/codegen.ts

# Run Config example
node --import @oxc-node/core/register examples/config.ts

# Run Rocket Lifecycle example
node --import @oxc-node/core/register examples/rocket-lifecycle.ts
```

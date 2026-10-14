# Rocket TypeScript User Manual (`@alloy-ts/server`)

`@alloy-ts/server` provides high-performance NAPI bindings to run the **Rocket** web framework directly from TypeScript and Node.js.

---

## Table of Contents

1. [Overview](#overview)
2. [Installation & Setup](#installation--setup)
3. [Alloy.serve Server API](#alloyserve-server-api)
4. [App Routes API](#app-routes-api)
5. [Response Handling](#response-handling)
6. [Static File Serving](#static-file-serving)
7. [Rocket Configuration](#rocket-configuration)
8. [Codegen & Type Utilities](#codegen--type-utilities)
9. [Running with `@oxc-node/core`](#running-with-oxc-nodecore)

---

## Overview

Rocket's core HTTP engine runs natively on a multithreaded Tokio runtime in Rust. JavaScript handlers are dispatched across threadsafe callback channels, giving Node.js applications Rocket's performance, safety, and request handling capabilities while preserving familiar TypeScript syntax.

---

## Installation & Setup

```bash
npm install @alloy-ts/server
```

---

## `Alloy.serve` Server API

`Alloy.serve` provides a Bun.serve-compatible HTTP server interface.

```ts
import { Alloy, Response } from "@alloy-ts/server";

const server = Alloy.serve({
  // Port resolves from process.env.BUN_PORT || PORT || NODE_PORT || 3000
  port: 8080,
  hostname: "0.0.0.0",

  routes: {
    // Static response
    "/api/status": new Response("OK"),

    // Dynamic path parameters (e.g. req.params.id)
    "/users/:id": (req) => {
      return new Response(`Hello User ${req.params.id}!`);
    },

    // Per-method handlers
    "/api/posts": {
      GET: () => new Response("List posts"),
      POST: async (req) => {
        const body = await req.json();
        return Response.json({ created: true, ...body });
      },
    },

    // Wildcard catch-all route
    "/api/*": Response.json({ message: "Not found" }, { status: 404 }),

    // Redirect response
    "/blog/hello": Response.redirect("/blog/hello/world"),

    // Lazily served file
    "/favicon.ico": Alloy.file("./favicon.ico"),
  },

  // Fallback for unmatched routes
  fetch(req) {
    return new Response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);

// Gracefully stop the server
server.stop();
```

---

## App Routes API

The `App` class re-expresses Rocket's `rocket::build()` builder pattern for TypeScript:

```ts
import { App, response } from "@alloy-ts/server";

const app = new App();

app.get("/", async () => response("Hello, world!"));
app.post("/items", async (req) => response("Item created", 201));

// Launch the server
const server = app.launch({ port: 3000 });
console.log(`Server running at ${server.url}`);

// Stop the server
app.stop();
```

---

## Response Handling

`Response` objects can be constructed directly or created with helper functions:

```ts
import { Response, response } from "@alloy-ts/server";

// Plain text response with status and custom headers
const r1 = new Response("Hello", {
  status: 200,
  headers: [["X-Custom", "Value"]],
});

// JSON response
const r2 = Response.json({ status: "success" });

// Redirect response
const r3 = Response.redirect("/login", 302);

// Shorthand helper function
const r4 = response("Quick response", 200);
```

---

## Static File Serving

Use `Alloy.file(path)` or `file(path)` to serve static files:

```ts
import { Alloy } from "@alloy-ts/server";

Alloy.serve({
  routes: {
    "/favicon.ico": Alloy.file("./public/favicon.ico"),
  },
});
```

---

## Rocket Configuration

Customize server workers, limits, shutdown behavior, and log levels:

```ts
import { Config } from "@alloy-ts/server";

// Load debug defaults
const config = Config.debugDefault();
config.port = 8000;
config.address = "127.0.0.1";
config.workers = 16;
config.logLevel = "normal";
config.limits = {
  limits: {
    json: "10MiB",
    form: "64KiB",
  },
};

// Create Config override object
const customCfg = Config.from({
  port: 9000,
  address: "0.0.0.0",
});
```

---

## Codegen & Type Utilities

`@alloy-ts/server` provides type-safe URI building and route metadata extraction:

```ts
import { uri, parseRoutePattern, generateRoutesTypeDefinitions } from "@alloy-ts/server";

// 1. Type-safe URI construction (supports :param and <param> syntax)
const profileUrl = uri("/users/:id/posts?<category>", {
  id: "123",
  category: "news",
});
// "/users/123/posts?category=news"

// 2. Route pattern parsing
const meta = parseRoutePattern("/items/<id>/<action..>?<filter>");
console.log(meta.pathParams); // [{ name: "id", isTrailing: false }, { name: "action", isTrailing: true }]
console.log(meta.queryParams); // ["filter"]

// 3. Generating TypeScript route interfaces
const tsCode = generateRoutesTypeDefinitions(["/users/:id", "/posts/:slug"]);
console.log(tsCode);
```

---

## Running with `@oxc-node/core`

Execute TypeScript server files directly without a pre-compilation step using `@oxc-node/core`:

```bash
node --import @oxc-node/core/register examples/app-routes.ts
```

Or run the npm package script:

```bash
npm run example
```

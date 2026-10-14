# Rocket Web Framework for TypeScript (`@alloy-ts/server`) - User Manual

This manual explains how to build high-performance web applications in TypeScript and Node.js powered by Rust's [Rocket web framework](https://rocket.rs/) via NAPI bindings.

---

## Quickstart

Install `@alloy-ts/server` in your Node.js or TypeScript project:

```bash
npm install @alloy-ts/server
```

To run TypeScript files directly without a separate build step, install `@oxc-node/core`:

```bash
npm install -D @oxc-node/core
```

---

## Basic Server Setup (`serve`)

The high-level `serve()` API provides a fast HTTP server interface backed by Rocket.

```ts
import { serve, Response, file } from "@alloy-ts/server";

const server = serve(
  {
    // Static routes
    "/api/status": new Response("OK"),

    // Dynamic routes
    "/users/:id": (req) => {
      return new Response(`Hello User ${req.params.id}!`);
    },

    // Per-HTTP method handlers
    "/api/posts": {
      GET: () => Response.json([{ id: 1, title: "First Post" }]),
      POST: async (req) => {
        const body = req.body ? JSON.parse(Buffer.from(req.body).toString()) : {};
        return Response.json({ created: true, ...body }, { status: 201 });
      },
    },

    // Redirect
    "/blog/hello": Response.redirect("/blog/hello/world"),

    // Serve a file from disk
    "/favicon.ico": file("./favicon.ico"),
  },

  // Fallback for unmatched routes
  (req) => {
    return new Response(`Not Found: ${req.path}`, { status: 404 });
  },

  {
    port: 3000,
    hostname: "127.0.0.1",
  },
);

console.log(`Server running at ${server.url}`);
```

---

## Rocket Builder API (`Rocket`)

For applications that prefer mounting routes on a Rocket instance:

```ts
import { Rocket, response } from "@alloy-ts/server";

const rocket = new Rocket({ port: 3000, address: "127.0.0.1" });

rocket.mount("/", "GET", "/", async () => response("Hello from Rocket!"));
rocket.mount("/users", "GET", "/:id", async (req: any) => response(`User ID: ${req.params.id}`));

// Launch the Rocket server
void rocket.launch();
```

---

## Route Registration & URI Generation

### Route Registration (`registerRoute`)

Register Rocket route handlers directly:

```ts
import { registerRoute, launch, response } from "@alloy-ts/server";

registerRoute("GET", "/users/:id", async (req: any) => response(`User ${req.params.id}`));
registerRoute("POST", "/users", async () => response("User created"));

void launch();
```

### Type-Safe URI Generation (`uri`)

Generates URL-encoded route URIs by replacing dynamic path variables:

```ts
import { uri } from "@alloy-ts/server";

const path1 = uri("/users/:id", { id: 42 });
// "/users/42"

const path2 = uri("/person/<name>", { name: "Mike Smith" });
// "/person/Mike%20Smith"
```

---

## Configuration (`Config`)

Customize Rocket's configuration options:

```ts
import { Config } from "@alloy-ts/server";

const config = Config.try_from({
  port: 8080,
  address: "0.0.0.0",
  workers: 8,
  logLevel: "debug",
  limits: {
    json: "10MiB",
    form: "64KiB",
  },
});

console.log(`Configured port: ${config.port}`);
```

---

## Execution

Run your TypeScript server file with `oxc-node`:

```bash
node --import @oxc-node/core server.ts
```

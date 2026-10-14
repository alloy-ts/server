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

## Basic Server Setup (`Alloy.serve`)

The high-level `Alloy.serve()` API provides a fast, Bun-compatible HTTP server interface backed by Rocket.

```ts
import { Alloy, Response } from "@alloy-ts/server";

const server = Alloy.serve({
  routes: {
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

    // Wildcard route for unmatched API requests
    "/api/*": Response.json({ message: "Not found" }, { status: 404 }),

    // Redirect
    "/blog/hello": Response.redirect("/blog/hello/world"),

    // Serve a file from disk
    "/favicon.ico": Alloy.file("./favicon.ico"),
  },

  // Fallback for unmatched routes
  fetch(req) {
    return new Response(`Not Found: ${req.path}`, { status: 404 });
  },

  port: 3000,
  hostname: "127.0.0.1",
});

console.log(`Server running at ${server.url}`);
```

---

## Express/Rocket Builder API (`App`)

For applications that prefer method chaining similar to Rocket's `rocket::build()` or Express:

```ts
import { App, response } from "@alloy-ts/server";

const app = new App();

app
  .get("/", async () => response("Hello from Rocket!"))
  .get("/users/:id", async (req) => response(`User ID: ${req.params.id}`))
  .post("/data", async (req) => response("Received"));

// Launch the Rocket server
app.launch();
```

---

## TypeScript Decorators & Codegen

Rocket's Rust procedural macros (`#[get]`, `routes!`, `uri!`) are represented in TypeScript via decorators and helper functions.

### Route Decorators

Decorate class methods to register Rocket route handlers:

```ts
import { Get, Post, Put, Delete, Catch, uri } from "@alloy-ts/server";

class UserController {
  @Get("/users/:id")
  getUser(req: any) {
    return `User ${req.params.id}`;
  }

  @Post("/users")
  createUser() {
    return "User created";
  }

  @Catch(404)
  notFound() {
    return "Custom 404";
  }
}
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
import { Config, Alloy } from "@alloy-ts/server";

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

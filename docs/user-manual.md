# @alloy-ts/server User Manual

`@alloy-ts/server` brings the Rust [Rocket web framework](https://rocket.rs/) to TypeScript and Node.js through high-performance NAPI bindings.

## Overview

Rocket's runtime engine (mounting, HTTP routing, multithreaded worker execution, configuration provider) executes natively in Rust. `@alloy-ts/server` bridges Rocket's semantics into JavaScript/TypeScript APIs.

---

## 1. Quickstart

### Basic Server (`Alloy.serve`)

```ts
import Alloy, { Response } from "@alloy-ts/server";

const server = Alloy.serve({
  port: 3000,
  routes: {
    // Static response
    "/api/status": new Response("OK"),

    // Dynamic path parameter
    "/users/:id": (req) => {
      return new Response(`Hello User ${req.params.id}!`);
    },

    // Per-method routes
    "/api/posts": {
      GET: () => Response.json([{ id: 1, title: "Hello Rocket" }]),
      POST: async (req) => {
        const body = req.body ? JSON.parse(req.body.toString()) : {};
        return Response.json({ created: true, ...body }, { status: 201 });
      },
    },

    // Wildcard route
    "/api/*": Response.json({ message: "Not found" }, { status: 404 }),

    // Redirect
    "/blog/hello": Response.redirect("/blog/hello/world"),

    // Lazy file serving
    "/favicon.ico": Alloy.file("./favicon.ico"),
  },

  // Fallback handler
  fetch(_req) {
    return new Response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);
```

---

## 2. App Builder Pattern (`App`)

The `App` class provides a 1:1 runtime counterpart to Rocket's `rocket::build()` and macro syntax:

| Rocket (Rust)          | `@alloy-ts/server` (TypeScript)         |
| ---------------------- | --------------------------------------- |
| `rocket::build()`      | `new App()`                             |
| `#[get("/")] fn hello` | `app.get("/", () => response("Hello"))` |
| `routes![hello]`       | (passed to `app.get` / `app.mount`)     |
| `#[launch] fn rocket`  | `app.launch()`                          |

### Example

```ts
import { App, response } from "@alloy-ts/server";

const app = new App();

app.get("/", async () => response("Hello, world!"));
app.get("/users/:id", async (req) => response(`User ${req.params.id}`));

app.launch({ port: 3000 });
```

---

## 3. Codegen & Decorators

In Rust, Rocket uses procedural macros (`#[get]`, `#[post]`, `uri!`, `#[catch]`). In TypeScript, `@alloy-ts/server` re-expresses these via TypeScript decorators and helpers:

### Route & Catcher Decorators

```ts
import { get, post, route, catchCatcher as catchRoute, Response, launch } from "@alloy-ts/server";

class UserController {
  @get("/users")
  listUsers() {
    return Response.json([]);
  }

  @post("/users")
  createUser() {
    return Response.json({ created: true }, { status: 201 });
  }

  @route("PUT", "/users/:id")
  updateUser() {
    return Response.json({ updated: true });
  }

  @catchRoute(404)
  notFound() {
    return new Response("Not Found", { status: 404 });
  }
}

new UserController();
launch();
```

### Type-Safe URI Generation (`uri`)

```ts
import { uri, buildRouteUri } from "@alloy-ts/server";

// Interpolates path parameters safely
const u1 = uri("/users/:id", { id: 42 }); // -> "/users/42"
const u2 = uri("/search/:query", { query: "rocket web framework" }); // -> "/search/rocket%20web%20framework"
```

---

## 4. Configuration (`Config`)

Rocket's configuration system is driven by Figment and mirrored directly in TypeScript via `Config`:

```ts
import { Config, serve, Response } from "@alloy-ts/server";

// Inspect defaults
const debugConfig = Config.debugDefault();
console.log(`Default debug port: ${debugConfig.port}`); // 8000

// Create custom config with partial overrides
const config = Config.tryFrom({
  port: 8080,
  address: "127.0.0.1",
  workers: 8,
  logLevel: "normal",
  shutdown: {
    grace: 5,
    mercy: 5,
    ctrlc: true,
  },
});

serve({
  port: config.port,
  hostname: config.address,
  fetch() {
    return new Response("Hello!");
  },
});
```

---

## 5. Running Code

Run TypeScript server files directly using `oxnode` or Node.js with `@oxc-node/core`:

```bash
npx oxnode examples/server.ts
```

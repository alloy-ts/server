# Rocket TypeScript / Node.js User Manual (`@alloy-ts/server`)

Welcome to the `@alloy-ts/server` User Manual. This library brings the speed, safety, and ergonomics of Rust's [Rocket v0.5](https://rocket.rs) web framework to Node.js and TypeScript via NAPI-RS bindings.

---

## Table of Contents

1. [Installation](#installation)
2. [Server Setup](#server-setup)
   - [Alloy.serve API](#alloyserve-api)
   - [App Router API](#app-router-api)
3. [Codegen & Decorators](#codegen--decorators)
4. [Request & Response API](#request--response-api)
   - [RequestInput](#requestinput)
   - [Response & ResponseInit](#response--responseinit)
   - [Static File Serving](#static-file-serving)
5. [Routes & Handlers](#routes--handlers)
6. [Configuration (`Config`)](#configuration-config)
7. [Type-Safe URI Helper (`uri`)](#type-safe-uri-helper-uri)

---

## Installation

```bash
npm install @alloy-ts/server
```

Requires Node.js 18+ and native support for your platform architecture.

---

## Server Setup

### Alloy.serve API

`Alloy.serve` (or `serve`) provides a high-performance server interface compatible with Bun / Alloy server options:

```ts
import { Alloy, Response } from "@alloy-ts/server";

const server = Alloy.serve({
  port: 3000,
  hostname: "0.0.0.0",
  routes: {
    // Static response
    "/api/health": new Response("OK"),

    // Dynamic handler with params
    "/users/:id": (req) => {
      return new Response(`User: ${req.params["id"]}`);
    },

    // Method map
    "/api/posts": {
      GET: () => Response.json([{ id: 1, title: "First Post" }]),
      POST: async (req) => {
        const body = req.body ? JSON.parse(req.body.toString()) : {};
        return Response.json({ created: true, ...body }, { status: 201 });
      },
    },

    // Redirect
    "/old-link": Response.redirect("/new-link"),

    // Static file response
    "/favicon.ico": Alloy.file("./favicon.ico"),
  },

  // Fallback handler for unmatched routes
  fetch(req) {
    return new Response(`Not Found: ${req.path}`, { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);
```

### App Router API

The `App` class translates Rocket's `rocket::build().mount(...).launch()` workflow directly into TypeScript:

```ts
import { App, response } from "@alloy-ts/server";

const app = new App();

app.get("/", async () => response("Hello World!"));
app.get("/users/:id", async (req) => response(`User: ${req.params["id"]}`));
app.post("/items", async (req) => response("Created", { status: 201 }));

// Launch the underlying Rocket instance
await app.launch();
```

---

## Codegen & Decorators

Rocket in Rust relies on procedural macros (`#[get]`, `#[post]`, etc.) to attach metadata to handler functions. `@alloy-ts/server` provides decorator utilities for TypeScript class-based controllers:

```ts
import { get, post, put, del, Response, launch } from "@alloy-ts/server";

class UserController {
  @get("/users")
  async listUsers() {
    return Response.json([{ id: 1, name: "Alice" }]);
  }

  @get("/users/:id")
  async getUser(req) {
    return Response.json({ id: req.params["id"] });
  }

  @post("/users")
  async createUser(req) {
    return Response.json({ created: true }, { status: 201 });
  }

  @del("/users/:id")
  async deleteUser(req) {
    return Response.json({ deleted: true });
  }
}

// Start the server with registered routes
await launch();
```

---

## Request & Response API

### RequestInput

Every dynamic route handler receives a `RequestInput` payload:

```ts
export interface RequestInput {
  method: string; // "GET", "POST", etc.
  path: string; // Path matching the request, e.g. "/users/123"
  query: string; // Query string, e.g. "page=1&sort=desc"
  headers: Array<[string, string]>; // Headers array
  body?: Buffer; // Capped raw body buffer (up to 1 MiB)
  params: Record<string, string>; // Parsed parameters, e.g. { id: "123" }
}
```

### Response & ResponseInit

`Response` encapsulates status code, HTTP headers, and raw body buffer:

```ts
import { Response } from "@alloy-ts/server";

// Plain response
const r1 = new Response("Text content", {
  status: 200,
  headers: { "X-Custom-Header": "value" },
});

// JSON response
const r2 = Response.json({ status: "success" }, { status: 200 });

// Redirect response
const r3 = Response.redirect("https://rocket.rs", 302);
```

### Static File Serving

Use `file(path)` or `Alloy.file(path)` to serve static files directly from disk:

```ts
import { Alloy } from "@alloy-ts/server";

Alloy.serve({
  routes: {
    "/index.html": Alloy.file("./public/index.html"),
    "/assets/logo.png": Alloy.file("./public/assets/logo.png"),
  },
});
```

---

## Routes & Handlers

Inspect or specify route metadata using the `Route` class:

```ts
import { Route } from "@alloy-ts/server";

const r = new Route("GET", "/users/:id", "get_user");
console.log(r.method, r.uri, r.rank); // "GET", "/users/:id", -5
```

---

## Configuration (`Config`)

Rocket's configuration system is exposed via the `Config` class:

```ts
import { Config } from "@alloy-ts/server";

// Seeded with Rocket's debug defaults
const config = new Config();
config.port = 8080;
config.address = "0.0.0.0";
config.workers = 8;
config.logLevel = "normal";

// Override with partial ConfigInit
const customConfig = Config.tryFrom({
  port: 9000,
  address: "127.0.0.1",
  limits: {
    limits: {
      json: "10MiB",
      form: "64KiB",
    },
  },
});

console.log(`Port: ${customConfig.port}, Limits:`, customConfig.limits);
```

---

## Type-Safe URI Helper (`uri`)

The `uri` helper function mirrors Rocket's `uri!` macro for path parameter interpolation:

```ts
import { uri } from "@alloy-ts/server";

const path1 = uri("/users/:id", { id: 42 });
// Output: "/users/42"

const path2 = uri("/posts/<slug>", { slug: "hello-world" });
// Output: "/posts/hello-world"
```

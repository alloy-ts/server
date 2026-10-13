# @alloy-ts/server

NAPI-RS bindings for the [Rocket web framework](https://rocket.rs) for TypeScript (Node.js).

## Overview

This library bridges Rocket's high-performance Rust web server runtime to TypeScript / Node.js and provides codegen support (decorators and type-safe URI helpers) for TypeScript applications.

## Features

- **High Performance:** Powered by Rocket v0.5 and Tokio async runtime.
- **TypeScript Codegen & Decorators:** Method decorators (`@get`, `@post`, `@put`, `@del`) to register routes.
- **Type-Safe URI Helpers:** `uri()` helper mirroring Rocket's `uri!` macro for path parameter interpolation (`:param` and `<param>`).
- **Web-standard Response API:** `Response` constructor, `Response.json()`, and `Response.redirect()`.
- **Alloy.serve API:** Compatibility with Bun/Alloy server options (`routes`, `fetch`, `port`, `hostname`).

## Quick Start

### Basic Server (`Alloy.serve`)

```ts
import { Alloy, Response } from "@alloy-ts/server";

const server = Alloy.serve({
  port: 3000,
  routes: {
    "/": new Response("Hello from Rocket!"),
    "/users/:id": (req) => new Response(`User ID: ${req.params.id}`),
    "/api/data": {
      GET: () => Response.json({ status: "ok" }),
      POST: async (req) => {
        return Response.json({ received: true });
      },
    },
  },
  fetch(req) {
    return new Response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);
```

### TypeScript Decorator Support (Codegen)

```ts
import { get, post, Response, launch } from "@alloy-ts/server";

class UserController {
  @get("/users/:id")
  async getUser(req) {
    return Response.json({ id: req.params.id });
  }

  @post("/users")
  async createUser(req) {
    return Response.json({ created: true }, { status: 201 });
  }
}

// Launch the Rocket server
await launch();
```

### Type-Safe URI Helper (`uri`)

```ts
import { uri } from "@alloy-ts/server";

const userUrl = uri("/users/:id", { id: 42 });
// "/users/42"

const postUrl = uri("/posts/<slug>", { slug: "hello-world" });
// "/posts/hello-world"
```

## Development

- Configure local hooks:

```bash
npm run prepare
```

- Install dependencies:

```bash
vp install
```

- Run the unit tests:

```bash
vp test
```

- Build the library:

```bash
npm run build
```

- Code formatting & linting:

```bash
npm run check
```

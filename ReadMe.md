# Rocket NAPI Bindings & Codegen for Node.js / TypeScript (`@alloy-ts/server`)

NAPI bindings that bridge the Rust [Rocket web framework](https://rocket.rs/) to Node.js and TypeScript, supporting high-performance HTTP serving and full code generation capabilities.

## Features

- **Rocket Web Framework Core**: Powered by Rocket in Rust via NAPI bindings with Tokiort.
- **TypeScript Codegen & Decorators**: `@Get`, `@Post`, `@Put`, `@Delete`, `@Catch`, `routes()`, `catchers()`, `FromForm`, and `Responder`.
- **Route Attribute Parser**: Parses Rocket route attribute grammar (`"/person/<name>?<age>", rank = 1, format = "json"`).
- **Type-Safe Route URI Builder**: `uri("/person/<name>?<age>", { name: "Mike", age: "28" })` URL-encodes and interpolates path and query parameters matching Rocket's `uri!` macro rules.
- **TypeScript Manifest Generator**: `generateTsRouteManifest()` generates TypeScript route manifests and type-safe URI helpers.

---

## Usage Example

### Basic Server

```ts
import { serve, Response, Config } from "@alloy-ts/server";

const server = serve({
  port: 8080,
  hostname: "127.0.0.1",
  routes: {
    "/": () => new Response("Hello from Rocket!"),
    "/api/users/:id": (req) => Response.json({ user: req.params.id }),
  },
});

console.log(`Server running on ${server.url}`);
```

### Route Decorators & Codegen

```ts
import { Get, Post, Catch, routes, catchers, uri, Response, FromForm } from "@alloy-ts/server";

class UserController {
  @Get("/users/<id>")
  getUser(req) {
    return Response.json({ id: req.params.id });
  }

  @Post("/users")
  async createUser(req) {
    const form = FromForm.parse(req.body?.toString() || "");
    return Response.json({ created: true, ...form });
  }

  @Catch(404)
  notFound() {
    return new Response("User not found", { status: 404 });
  }
}

// Generate URI matching Rocket's uri! macro rules
const userUri = uri("/users/<id>", { id: "42" }); // "/users/42"

// Collect routes & catchers
const routeTable = routes(new UserController());
const catcherList = catchers(new UserController());
```

---

## Development

- Install dependencies:

```bash
npm install
```

- Build NAPI bindings and TypeScript exports:

```bash
npm run build:napi
```

- Run unit tests:

```bash
npm test
```

- Code check and formatting:

```bash
npm run check
```

# @alloy-ts/server

Rocket Web Framework NAPI bindings for Node.js and TypeScript.

This package bridges Rocket's high-performance Rust web framework to TypeScript and Node.js using N-API (`@napi-rs/cli`).

## Features

- **`Alloy.serve`**: High-performance HTTP server powered by Rocket's runtime.
- **Route Handlers**: Dynamic paths (`/users/:id`), catch-all routes (`/api/*`), and per-HTTP method handlers (`GET`, `POST`, etc.).
- **Codegen & Type-Safe URIs**: `Alloy.uri("/users/:id", { id: 42 })` for type-safe route URI construction.
- **Response Builders**: `Alloy.Response.json()`, `Alloy.Response.redirect()`, and custom status/headers.
- **Static Files**: `Alloy.file("./filepath")` for serving files directly.

## Usage

```ts
import { Alloy } from "@alloy-ts/server";

const server = Alloy.serve({
  port: 3000,
  routes: {
    "/": new Alloy.Response("Hello World!"),
    "/users/:id": (req) => {
      return new Alloy.Response(`Hello User ${req.params.id}!`);
    },
    "/api/posts": {
      GET: () => Alloy.Response.json([{ id: 1, title: "Post 1" }]),
      POST: async (req) => {
        return Alloy.Response.json({ created: true });
      },
    },
    "/favicon.ico": Alloy.file("./favicon.ico"),
  },
});

console.log(`Server running at ${server.url}`);
```

### Type-Safe URI Generation (Codegen Helper)

```ts
import { Alloy } from "@alloy-ts/server";

const profileUri = Alloy.uri("/users/:id/posts/:post_id", {
  id: "42",
  post_id: "hello world",
});
// Outputs: "/users/42/posts/hello%20world"
```

## Development

- Install dependencies:

```bash
vp install
```

- Build NAPI native binary:

```bash
npm run build:napi
```

- Run unit tests:

```bash
npm test
```

- Code check & formatting:

```bash
npm run check
```

# Alloy Server - Rocket NAPI Bindings & Codegen

High-performance NAPI bindings bridging Rust's [Rocket web framework](https://rocket.rs/) to TypeScript and Node.js with code generation (codegen) support.

## Overview

In Rust, Rocket's code generation (`#[get]`, `#[post]`, `routes!`, `uri!`, `derive(FromForm)`) relies on procedural macros evaluated at compile-time. This package provides the Node.js / TypeScript equivalent:

1. **NAPI Binding Layer**: Bridges Rocket's multi-threaded Tokio runtime, server lifecycle (`Rocket`, `Config`), request handling, and response construction into Node.js.
2. **TypeScript Codegen & Decorators**: Offers `@get`, `@post`, `@put`, `@del`, `@patch`, and `@catchError` decorators, route pattern translation (`:param` <-> `<param>`), and static type generation for type-safe `uri()` builders.
3. **Alloy Server DSL**: Declarative Bun/Alloy-style `Alloy.serve({ routes, fetch, port, hostname })` server initialization.

## Usage Example

### Basic Server with `Alloy.serve`

```ts
import { Alloy, response } from "@alloy-ts/server";

const server = Alloy.serve({
  port: 3000,
  routes: {
    "/": "Hello from Rocket!",
    "/api/status": response("OK", { status: 200 }),
    "/users/:id": (req) => {
      return response(`Hello User ${req.params.id}!`);
    },
  },
});

console.log(`Server running at ${server.url}`);
```

### TypeScript Decorators

```ts
import { get, post, catchError } from "@alloy-ts/server";

class UserController {
  @get("/users/:id")
  getUser(req) {
    return { id: req.params.id };
  }

  @post("/users")
  createUser(req) {
    return { created: true };
  }

  @catchError(404)
  notFound(req) {
    return { error: "Not Found" };
  }
}
```

### Type-Safe Route URI Code Generation

```ts
import { generateTsUriBuilder, parseRouteUri } from "@alloy-ts/server";

const tsCode = generateTsUriBuilder([
  { name: "getUser", method: "GET", path: "/users/:id" },
  { name: "getPost", method: "GET", path: "/posts/:slug" },
]);

// Generated code:
// export const uri = {
//   getUser: (id: string | number) => `/users/${id}`,
//   getPost: (slug: string | number) => `/posts/${slug}`,
// };
```

## Development

- Install dependencies:

```bash
npm install
```

- Build the NAPI native binding & JS library:

```bash
npm run build
```

- Run tests (Vitest + Node test runner):

```bash
npm run test
```

- Code check (formatting & linting):

```bash
npm run check
```

- Code formatting:

```bash
npm run fmt
```

- Linting:

```bash
npm run lint
```

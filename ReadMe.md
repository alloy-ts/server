# `@alloy-ts/server`

NAPI bindings for the **Rocket** web framework in TypeScript and Node.js with codegen utilities.

---

## Features

- **`Alloy.serve` HTTP Server**: High-performance HTTP server API powered by a native Rust / Rocket backend on a multithreaded Tokio runtime.
- **Flexible Routing**:
  - Static routes (`/api/status`)
  - Dynamic routes with named parameters (`/users/:id` -> `req.params.id`)
  - Per-HTTP method handlers (`GET`, `POST`, `PUT`, `DELETE`, etc.)
  - Wildcard routes (`/api/*`)
  - Redirect responses (`Response.redirect("/blog/hello/world")`)
  - Static file serving (`Alloy.file("./favicon.ico")`)
  - Fallback request handler (`fetch(req)`)
- **Code Generation & Macro Utilities**:
  - `uri("/users/:id", { id: "42" })`: Type-safe URL constructor mirroring Rocket's `uri!` macro.
  - `parseRoutePattern(pattern)`: Parses route patterns into structured parameter metadata.
  - `generateTsRouteTypes(routes)`: Generates TypeScript interface definitions for API routes.
  - Route decorators (`@get`, `@post`, `@put`, `@del`, `@route`) and metadata tracking (`getRegisteredRoutesMeta`).
- **Rocket Configuration System**:
  - Mirror of Rocket's Figment-based `Config` struct (`debugDefault`, `releaseDefault`, `tryFrom`, `from`).

---

## Quick Start

```ts
import { Alloy, Response } from "@alloy-ts/server";

const server = Alloy.serve({
  routes: {
    // Static route
    "/api/status": new Response("OK"),

    // Dynamic route with parameter
    "/users/:id": (req) => {
      return new Response(`Hello User ${req.params.id}!`);
    },

    // Per-HTTP method handlers
    "/api/posts": {
      GET: () => new Response("List posts"),
      POST: async (req) => {
        const body = await req.json();
        return Response.json({ created: true, ...body });
      },
    },

    // Wildcard route
    "/api/*": Response.json({ message: "Not found" }, { status: 404 }),

    // Redirect response
    "/blog/hello": Response.redirect("/blog/hello/world"),

    // Serve static file
    "/favicon.ico": Alloy.file("./favicon.ico"),
  },

  // Fallback for unmatched routes
  fetch(req) {
    return new Response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);
```

---

## Codegen & Type Generation

```ts
import { uri, parseRoutePattern, generateRoutesTypeDefinitions } from "@alloy-ts/server";

// Type-safe URI building
const url = uri("/users/:id", { id: "42" });
// "/users/42"

// Generate TypeScript definitions for route patterns
const tsDefs = generateRoutesTypeDefinitions(["/api/users/:id"]);
console.log(tsDefs);
// export interface ApiRoutes { ... }
```

---

## Configuration

```ts
import { Config } from "@alloy-ts/server";

const cfg = Config.debugDefault();
cfg.port = 8080;
cfg.address = "0.0.0.0";
cfg.limits = { limits: { json: "2MiB", form: "64KiB" } };
```

---

## Development

- Install dependencies:

```bash
vp install
```

- Build native bindings and package:

```bash
npm run build
```

- Run unit tests:

```bash
npm run test
```

- Code formatting & lint check:

```bash
npm run check
```

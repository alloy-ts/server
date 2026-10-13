# @alloy-ts/server

TypeScript & Node.js NAPI bindings for the [Rocket](https://rocket.rs) web framework.

This library brings Rocket's fast, type-safe web server runtime to Node.js and TypeScript, supporting route routing, Rocket configuration, decorator-based codegen, and type-safe URI generation.

## Features

- **Rocket Web Server Runtime**: Driven by Rocket v0.5 running on a multi-threaded Tokio runtime.
- **`Alloy.serve` API**: Simple route dictionary or `fetch` fallback serving.
- **Decorators**: `@Get`, `@Post`, `@Put`, `@Delete`, `@Patch`, `@Head`, `@Options`, `@Route`, and `@Catch`.
- **Response Ergonomics**: `Response`, `Response.json()`, and `Response.redirect()`.
- **Type-safe URI Generator**: `uri()` and `buildRouteUri()` for parameterized route URIs (`:id`, `<id>`, `<id..>`).
- **TypeScript Codegen Utilities**: `parseRoutePattern()` and `generateRoutesTypeDefinitions()` for generating TypeScript route interface definitions.

## Usage

### Server Setup with `Alloy.serve`

```typescript
import { Alloy, Response } from "@alloy-ts/server";

const server = Alloy.serve({
  port: 8080,
  routes: {
    "/": new Response("Hello from Rocket!"),
    "/users/:id": (req) => Response.json({ id: req.params.id }),
  },
  fetch(req) {
    return new Response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);
```

### Route Decorators

```typescript
import { Get, Post, Response } from "@alloy-ts/server";

class UserController {
  @Get("/users/:id")
  getUser(req: any) {
    return Response.json({ id: req.params.id });
  }

  @Post("/users")
  createUser() {
    return new Response("Created", { status: 201 });
  }
}
```

### Type-Safe URI Generation

```typescript
import { uri, buildRouteUri } from "@alloy-ts/server";

// Replace path parameters
const userUrl = uri("/users/:id", { id: "42" });
// -> "/users/42"

// Build route URI with query parameters
const fullUrl = buildRouteUri("/users/<id>?<ref>", { id: "10", ref: "campaign" });
// -> "/users/10?ref=campaign"
```

### Route Codegen Utilities

```typescript
import { generateRoutesTypeDefinitions } from "@alloy-ts/server";

const tsDefs = generateRoutesTypeDefinitions([
  "/api/users/<id>",
  "/api/posts/<id>/<action..>?<ref>",
]);

console.log(tsDefs);
// Outputs TypeScript interface ApiRoutes with path and query parameter types
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

- Run unit tests:

```bash
npm test
```

- Build native bindings and TypeScript library:

```bash
npm run build
```

- Code formatting:

```bash
npm run fmt
```

- Linting and type checking:

```bash
npm run check
```

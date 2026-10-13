# Server

Node.js (TypeScript) NAPI bindings for the Rocket web framework and code generation utilities.

## Features

- **High-performance HTTP server (`serve`)**: Bun/Alloy-style routing backed by Rocket.
- **TypeScript Code Generation (`codegen`)**:
  - `parseRoute` / `parseRoutes`: Resolve route specifications into normalized `RouteDef` structures with extracted path and query parameters.
  - `toRocketPath`: Translate route syntax into Rocket's `<param>` path syntax.
  - `generateTypes`: Generate TypeScript interfaces and type aliases (`RouteTypes`, `ParamsOf`, `QueryOf`, `BodyOf`, `ResponseOf`, `RouteContext`).
  - `generateClient`: Generate a strongly-typed fetch client SDK (`RocketClient`) and `uri()` URL builder.
  - `generateModule`: Generate a complete TypeScript module containing `ROUTES`, type declarations, `uri()` helper, and `RocketClient`.
  - `httpMethods`: Get the array of HTTP methods supported by Rocket.

---

## Quick Start

### Basic Server (`serve`)

```ts
import { serve, response, file } from "@alloy-ts/server";

const server = serve({
  port: 3000,
  routes: {
    "/api/status": response("OK"),
    "/users/:id": (req) => response(`Hello ${req.params?.id}`),
    "/api/posts": {
      GET: () => response("List posts"),
      POST: async (req) => response.json({ created: true }),
    },
    "/favicon.ico": file("./favicon.ico"),
  },
  fetch(req) {
    return response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);
```

---

## Code Generation (Codegen)

### Route Parsing & Path Translation

```ts
import { parseRoute, toRocketPath } from "@alloy-ts/server";

const route = parseRoute({
  method: "GET",
  path: "/users/:id?<active>",
  response: "User",
});

console.log(route);
// {
//   key: "GET /users/:id",
//   method: "GET",
//   path: "/users/:id",
//   rocketPath: "/users/<id>",
//   name: "getUsersById",
//   params: [{ name: "id", tsName: "id", kind: "Path", tsType: "string | number", trailing: false, optional: false }],
//   query: [{ name: "active", tsName: "active", kind: "Query", tsType: "string | number", trailing: false, optional: true }],
//   response: "User"
// }

const rocketPath = toRocketPath("/users/:id/*");
// "/users/<id>/<_..>"
```

### TypeScript Client SDK & Type Generation

```ts
import { parseRoutes, generateModule } from "@alloy-ts/server";

const routes = parseRoutes([
  { method: "GET", path: "/users/:id", response: "User" },
  { method: "POST", path: "/users", body: "CreateUser", response: "User" },
]);

const code = generateModule(routes, {
  clientName: "UserApiClient",
  baseUrl: "http://localhost:3000",
});

console.log(code);
```

---

## Development

- Configure local hooks:

```bash
npm run prepare
```

- Install dependencies:

```bash
npm install
```

- Run unit tests:

```bash
npm test
```

- Build library & native NAPI bindings:

```bash
npm run build
```

- Code formatting:

```bash
npm run fmt
```

- Linting:

```bash
npm run lint
```

- Code check:

```bash
npm run check
```

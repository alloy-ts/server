# Server

Node.js (TypeScript) NAPI bindings for the Rocket web framework and code generation utilities.

## Features

- **High-performance HTTP server (`Alloy.serve` / `serve`)**: Bun/Alloy-style routing backed by Rocket.
- **Rocket Server Lifecycle (`Rocket`)**: Direct programmatic control over mounting routes, registering catchers, ignition, and graceful shutdown.
- **Rocket Configuration (`Config`)**: Full mirror of Rocket's server configuration (`Config.debugDefault()`, `Config.tryFrom()`, limits, shutdown signals, log levels).
- **TypeScript Code Generation (`codegen`)**:
  - `parseRoute` / `parseRoutes`: Resolve Rocket / Alloy route patterns into structured route definitions (`RouteDef`).
  - `toRocketPath`: Translate Alloy/Bun route syntax (`/users/:id/*`) into Rocket route syntax (`/users/<id>/<_..>`).
  - `generateTypes`: Generate TypeScript route map interface and type utilities (`ParamsOf`, `QueryOf`, `RouteHandler`, etc.).
  - `generateClient`: Generate a strongly-typed `fetch` client class and `uri()` URL builder.
  - `generateModule`: Generate a complete module containing route constants, route map types, `uri()` helper, and typed client SDK.

---

## Quick Start

### Basic Server (`Alloy.serve`)

```ts
import { serve, Response, file } from "@alloy-ts/server";

const server = serve({
  port: 3000,
  routes: {
    "/api/status": new Response("OK"),
    "/users/:id": (req) => new Response(`Hello ${req.params?.id}`),
    "/api/posts": {
      GET: () => new Response("List posts"),
      POST: async (req) => Response.json({ created: true }),
    },
    "/favicon.ico": file("./favicon.ico"),
  },
  fetch(req) {
    return new Response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);
```

---

## Code Generation (Codegen)

### Route Pattern Parsing & Translation

```ts
import { parseRoute, parseRoutes, toRocketPath } from "@alloy-ts/server";

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

const rocketPattern = toRocketPath("/users/:id/*");
// "/users/<id>/<_..>"
```

### TypeScript Client SDK & Module Generation

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

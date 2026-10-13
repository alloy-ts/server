# Server

Node.js (TypeScript) NAPI bindings for the Rocket web framework and code generation utilities.

## Features

- **High-performance HTTP server (`serve`)**: Bun/Alloy-style routing backed by Rocket.
- **Rocket Server Lifecycle (`Rocket`)**: Direct programmatic control over mounting routes, registering catchers, ignition, and graceful shutdown.
- **Rocket Configuration (`Config`)**: Full mirror of Rocket's server configuration (`Config.debugDefault()`, `Config.tryFrom()`, limits, shutdown signals, log levels).
- **TypeScript Code Generation (`codegen`)**:
  - `parseRoute` / `parseRoutes`: Resolve dynamic path and query parameters into route definitions.
  - `toRocketPath`: Translate route patterns (`/users/:id/*`) into Rocket route syntax (`/users/<id>/<_..>`).
  - `generateTypes`: Generate TypeScript type declarations (`RouteTypes`, `ParamsOf`, `QueryOf`, etc.).
  - `generateClient`: Generate a strongly-typed fetch client SDK.
  - `generateModule`: Emit a complete module with route maps, helpers, and typed client SDK.

---

## Quick Start

### Basic Server (`serve`)

```ts
import { serve, response, responseJson, file } from "@alloy-ts/server";

const server = serve(
  {
    "/api/status": response("OK"),
    "/users/:id": async (req) => response(`Hello ${req.params.id}`),
    "/api/posts": {
      GET: async () => response("List posts"),
      POST: async (req) => responseJson({ created: true }),
    },
    "/favicon.ico": file("./favicon.ico"),
  },
  async (req) => response("Not Found", { status: 404 }),
  { port: 3000 },
);

console.log(`Server running at ${server.url}`);
```

---

## Code Generation (Codegen)

### Route Pattern Parsing & Translation

```ts
import { parseRoute, toRocketPath } from "@alloy-ts/server";

const route = parseRoute({
  method: "POST",
  path: "/users/<id..>?<q>&closed",
  body: "CreateUser",
  response: "User",
});
// Extracted route definition:
// {
//   key: "POST /users/:id",
//   method: "POST",
//   path: "/users/:id",
//   rocketPath: "/users/<id..>",
//   name: "postUsersById",
//   params: [ { name: "id", tsName: "id", kind: "Path", tsType: "string | number", trailing: true, optional: true } ],
//   query: [ ... ],
//   body: "CreateUser",
//   response: "User"
// }

const rocketPath = toRocketPath("/users/:id/*");
// "/users/<id>/<_..>"
```

### TypeScript Client SDK Generation

```ts
import { parseRoute, generateClient } from "@alloy-ts/server";

const route = parseRoute({
  method: "GET",
  path: "/users/:id",
  response: "User",
});

const clientCode = generateClient([route], {
  clientName: "UserApiClient",
  baseUrl: "https://api.example.com",
});

console.log(clientCode);
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

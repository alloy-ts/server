# Server

Node.js (TypeScript) NAPI bindings for the Rocket web framework and code generation utilities.

## Features

- **High-performance HTTP server (`Alloy.serve` / `serve`)**: Bun/Alloy-style routing backed by Rocket.
- **Rocket Server Lifecycle (`Rocket`)**: Direct programmatic control over mounting routes, registering catchers, ignition, and graceful shutdown.
- **Rocket Configuration (`Config`)**: Full mirror of Rocket's server configuration (`Config.debugDefault()`, `Config.tryFrom()`, limits, shutdown signals, log levels).
- **TypeScript Code Generation (`codegen`)**:
  - `parseRocketRoutePattern`: Extract dynamic path/query parameters from Rocket route strings.
  - `translatePatternToAlloy`: Translate Rocket route syntax (`/users/<id>`) into Alloy/Bun route syntax (`/users/:id`).
  - `generateTypescriptClient`: Generate a strongly-typed fetch client SDK for Rocket route specifications.
  - `generateRouteHandler`: Generate TypeScript route handler boilerplate.
  - `generateTypescriptInterface`: Convert Rust struct/enum field definitions into TypeScript interfaces.

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
import { parseRocketRoutePattern, translatePatternToAlloy } from "@alloy-ts/server";

const params = parseRocketRoutePattern("/users/<id>?<age>&<name..>");
// Extracted parameters:
// [
//   { name: "id", paramType: "string", isPath: true, isQuery: false, isOptional: false },
//   { name: "age", paramType: "string", isPath: false, isQuery: true, isOptional: false },
//   { name: "name", paramType: "string", isPath: false, isQuery: true, isOptional: true }
// ]

const alloyPattern = translatePatternToAlloy("/users/<id>/<rest..>");
// "/users/:id/*"
```

### TypeScript Client SDK Generation

```ts
import { generateTypescriptClient } from "@alloy-ts/server";

const result = generateTypescriptClient(
  [
    {
      method: "GET",
      path: "/users/<id>",
      name: "getUser",
      params: [
        { name: "id", paramType: "string", isPath: true, isQuery: false, isOptional: false },
      ],
      returnType: "Promise<{ id: string; name: string }>",
    },
  ],
  { clientName: "UserApiClient" },
);

console.log(result.code);
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

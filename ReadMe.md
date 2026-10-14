# `@alloy-ts/server`

Node.js (TypeScript) NAPI bindings for the [Rocket web framework](https://rocket.rs) and code generation utilities.

---

## Features

- **High-performance HTTP server (`serve`)**: Bun/Alloy-style routing backed by Rocket's HTTP runtime.
- **Request Helpers (`reqText`, `reqJson`)**: Native request body parsing for UTF-8 text and JSON.
- **Handler Outcomes (`HandlerOutcome`)**: Native outcome support (`success`, `error`, `forward`).
- **Rocket Server Lifecycle (`Rocket`)**: Direct programmatic control over mounting routes, registering catchers, ignition, launch, and graceful shutdown.
- **Rocket Configuration (`Config`)**: Full mirror of Rocket's server configuration (`Config.debugDefault()`, `Config.tryFrom()`, limits, shutdown signals, log levels).
- **TypeScript Code Generation (`codegen`)**:
  - `parseRocketRoutePattern`: Extract dynamic path/query parameters from Rocket route strings.
  - `translatePatternToAlloy`: Translate Rocket route syntax (`/users/<id>`) into Alloy/Bun route syntax (`/users/:id`).
  - `generateTypescriptClient`: Generate a strongly-typed fetch client SDK for Rocket route specifications.
  - `generateRouteHandler`: Generate TypeScript route handler boilerplate.
  - `generateTypescriptInterface`: Convert Rust struct/enum field definitions into TypeScript interfaces.

---

## Quick Start

### Basic Server (`serve`)

```ts
import {
  serve,
  response,
  responseJson,
  responseRedirect,
  file,
  reqText,
  reqJson,
} from "@alloy-ts/server";

const server = serve({
  routes: {
    "/api/status": response("OK"),
    "/users/:id": (req) => response(`Hello User ${req.params?.id}`),
    "/api/echo": async (req) => responseJson({ text: reqText(req.body) }),
    "/api/posts": {
      GET: () => response("List posts"),
      POST: async (req) => responseJson({ body: reqJson(req.body) }),
    },
    "/blog/hello": responseRedirect("/blog/hello/world"),
    "/favicon.ico": file("./favicon.ico"),
  },
  fetch(_req) {
    return response("Not Found", { status: 404 });
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
      returnType: "Promise<{ id: string; name: string }>",
    },
  ],
  { clientName: "UserApiClient", baseUrl: "https://api.example.com" },
);

console.log(result.code);
```

---

## Documentation

- [User Manual](docs/user-manual.md)
- [Implementation Status](docs/implementation-status.md)
- Examples: [examples/server.ts](examples/server.ts), [examples/app-routes.ts](examples/app-routes.ts)

---

## Development

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

- Code check (formatting, linting, type checks):

```bash
npm run check
```

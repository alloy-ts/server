# Server

Node.js (TypeScript) NAPI bindings for the Rocket web framework and code generation utilities.

## Features

- **High-performance HTTP server (`Alloy.serve` / `serve`)**: Bun/Alloy-style routing backed by Rocket supporting options objects, environment port fallbacks (`$BUN_PORT`, `$PORT`, `$NODE_PORT`), dynamic port selection (`port: 0`), HTML string/import routes, wildcards, and redirects.
- **Rocket Server Lifecycle (`Rocket`)**: Direct programmatic control over mounting routes, registering catchers, ignition, and graceful shutdown.
- **Rocket Configuration (`Config`)**: Full mirror of Rocket's server configuration (`Config.debugDefault()`, `Config.tryFrom()`, limits, shutdown signals, log levels).
- **TypeScript Code Generation (`codegen`)**:
  - `parseRoute` / `parseRoutes`: Resolve route specifications and dynamic parameters into structured route definitions.
  - `toRocketPath`: Translate route patterns (`/users/:id/*`) into Rocket syntax (`/users/<id>/<_..>`).
  - `generateTypes`: Emit TypeScript type definitions and `RouteHandler` signatures.
  - `generateClient`: Emit a strongly-typed `fetch` client SDK.
  - `generateModule`: Emit a complete TypeScript module combining `ROUTES`, route map types, `uri()` builder, and client.

---

## Quick Start

### Basic Server (`Alloy.serve`)

```ts
import { Alloy, response, responseJson, responseRedirect, file } from "@alloy-ts/server";

const server = Alloy.serve({
  port: 3000,
  routes: {
    // Static routes
    "/api/status": response("OK"),

    // Dynamic routes
    "/users/:id": (req) => response(`Hello User ${req.params.id}!`),

    // Per-HTTP method handlers
    "/api/posts": {
      GET: () => response("List posts"),
      POST: async (req) => responseJson({ created: true }),
    },

    // Wildcard route
    "/api/*": responseJson({ message: "Not found" }, { status: 404 }),

    // Redirects & Static Files
    "/blog/hello": responseRedirect("/blog/hello/world"),
    "/favicon.ico": file("./favicon.ico"),
  },

  fetch(req) {
    return responseJson({ error: "Not Found", path: req.path }, { status: 404 });
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

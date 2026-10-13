# Server

Node.js (TypeScript) NAPI bindings for the Rocket web framework and code generation utilities.

## Features

- **High-performance HTTP server (`Alloy.serve` / `serve`)**: Bun/Alloy-style routing backed by Rocket.
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
import { serve, response, responseJson, responseRedirect, file } from "@alloy-ts/server";

const server = serve(
  {
    // Static response
    "/api/status": response("OK"),

    // Dynamic parameter route
    "/users/:id": async (req) => response(`Hello User ${req.params.id}!`),

    // Per-HTTP method handlers
    "/api/posts": {
      GET: async () => response("List posts"),
      POST: async (req) => responseJson({ created: true }),
    },

    // Wildcard route
    "/api/*": responseJson({ message: "Not found" }, { status: 404 }),

    // Redirect
    "/blog/hello": responseRedirect("/blog/hello/world"),

    // File serving
    "/favicon.ico": file("./favicon.ico"),
  },

  // (optional) fallback for unmatched routes
  async (req) => response("Not Found", { status: 404 }),

  // Configuration options (port, hostname, unix)
  { port: 3000, hostname: "0.0.0.0" },
);

console.log(`Server running at ${server.url}`);
```

### Server Configuration Options

- **Port and Hostname**: Set `port` and `hostname` in options. `port: 0` randomly selects an available free port. Port defaults to `$BUN_PORT`, `$PORT`, or `$NODE_PORT` environment variables before falling back to `3000`.
- **Unix Domain Sockets**: Pass `unix: "/tmp/my-socket.sock"` (or `"\0abstract-socket"` for abstract namespace sockets on Linux) to listen on a Unix socket.

```ts
const server = serve({}, async (req) => response("Not Found", { status: 404 }), {
  port: 0,
  hostname: "127.0.0.1",
});

console.log(`Bound to random port ${server.port} at ${server.url}`);
```

---

## Rocket Configuration (`Config`)

```ts
import { Config } from "@alloy-ts/server";

// Seeded with Rocket's debug defaults
const config = Config.debugDefault();
config.port = 8080;
config.address = "0.0.0.0";
config.logLevel = "normal";
config.limits = { limits: { json: "10MiB", form: "64KiB" } };
config.shutdown = { ctrlc: true, grace: 5, mercy: 5, signals: ["term", "int"] };

// Create config from partial overrides
const customConfig = Config.tryFrom({
  port: 9000,
  address: "127.0.0.1",
  workers: 8,
  logLevel: "critical",
});
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

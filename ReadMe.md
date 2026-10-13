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
  // Port defaults to $BUN_PORT, $PORT, $NODE_PORT or 3000. Set port: 0 for dynamic random port.
  port: 3000,
  hostname: "127.0.0.1",

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

    // Wildcard route for all unmatched /api/* paths
    "/api/*": responseJson({ message: "Not found" }, { status: 404 }),

    // Redirects & Static Files
    "/blog/hello": responseRedirect("/blog/hello/world"),
    "/favicon.ico": file("./favicon.ico"),

    // HTML string or import routes
    "/": "<h1>Welcome to Alloy Web Server!</h1>",
  },

  fetch(req) {
    return responseJson({ error: "Not Found", path: req.path }, { status: 404 });
  },
});

console.log(`Server running at ${server.url} (port: ${server.port})`);
```

### Examples & Demos

Check out `examples/server.ts` for a complete runnable demo:

```bash
npx tsx examples/server.ts
```

---

## Configuration

### Rocket Configuration (`Config`)

```ts
import { Config } from "@alloy-ts/server";

// Get debug defaults or customize options
const config = Config.tryFrom({
  port: 8080,
  address: "127.0.0.1",
  workers: 16,
  logLevel: "normal",
  shutdown: {
    ctrlc: true,
    grace: 5,
    mercy: 5,
  },
});

console.log(`Configured port: ${config.port}`);
```

---

## Code Generation (Codegen)

### Route Pattern Parsing & Translation

```ts
import { parseRoute, toRocketPath } from "@alloy-ts/server";

const routeDef = parseRoute({
  method: "GET",
  path: "/users/:id?<age>",
  response: "User",
});

const rocketPath = toRocketPath("/users/:id/*");
// "/users/<id>/<_..>"
```

### TypeScript Client SDK Generation

```ts
import { parseRoutes, generateModule } from "@alloy-ts/server";

const routes = parseRoutes([
  { method: "GET", path: "/users/:id", response: "User" },
  { method: "POST", path: "/users", body: "CreateUser", response: "User" },
]);

const code = generateModule(routes, { baseUrl: "http://localhost:3000" });
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

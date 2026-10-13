# Server

Node.js (TypeScript) NAPI bindings for the Rocket web framework and code generation utilities.

## Features

- **High-performance HTTP server (`Alloy.serve` / `Bun.serve` / `serve`)**: Bun/Alloy-style HTTP server API backed by Rocket.
  - **Static & Dynamic Routing**: Supports static routes, dynamic segments (`/users/:id`), per-method handler maps (`GET`, `POST`, ...), and wildcard catch-alls (`/api/*`).
  - **Redirects & File Serving**: First-class `Response.redirect()` and `Alloy.file()` / `Bun.file()` static file serving.
  - **Port & Socket Configuration**: Configurable port and hostname, fallback port selection via `$BUN_PORT`, `$PORT`, `$NODE_PORT` (defaulting to `3000`), random port allocation (`port: 0`), and Unix domain / abstract namespace socket binding (`unix: "/tmp/my-socket.sock"`).
- **Rocket Server Lifecycle (`Rocket`)**: Programmatic control over mounting routes, registering catchers, ignition, and graceful shutdown.
- **Rocket Configuration (`Config`)**: Full mirror of Rocket's server configuration (`Config.debugDefault()`, `Config.releaseDefault()`, `Config.tryFrom()`, limits, shutdown signals, log levels).
- **TypeScript Code Generation (`codegen`)**:
  - `parseRoute` / `parseRoutes`: Extract dynamic path/query parameters and build route definitions.
  - `toRocketPath`: Translate Alloy/Bun route syntax (`/users/:id/*`) into Rocket route syntax (`/users/<id>/<_..>`).
  - `generateClient` / `generateModule`: Generate a strongly-typed fetch client SDK and route interfaces for Rocket route specifications.

---

## Quick Start

### Basic Server (`Alloy.serve`)

```ts
import { Alloy, Response } from "@alloy-ts/server";

const server = Alloy.serve({
  routes: {
    // Static routes
    "/api/status": new Response("OK"),

    // Dynamic routes
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

    // Redirects
    "/blog/hello": Response.redirect("/blog/hello/world"),

    // Serve a file lazily
    "/favicon.ico": Alloy.file("./favicon.ico"),
  },

  // Fallback for unmatched routes
  fetch(req) {
    return new Response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);
```

### Server Configuration

#### Changing Port and Hostname

```ts
const server = Alloy.serve({
  port: 8080, // Defaults to $BUN_PORT, $PORT, $NODE_PORT or 3000
  hostname: "0.0.0.0",
  fetch(req) {
    return new Response("404!");
  },
});

console.log(server.url); // http://localhost:8080
```

#### Random Port Assignment

To randomly select an available port, set `port: 0`:

```ts
const server = Alloy.serve({
  port: 0, // Random available port assigned by OS
  fetch(req) {
    return new Response("OK");
  },
});

console.log(server.port); // e.g. 42391
```

#### Unix Domain Sockets & Abstract Sockets

```ts
// Unix domain socket
Alloy.serve({
  unix: "/tmp/my-socket.sock",
  fetch(req) {
    return new Response("OK");
  },
});

// Abstract namespace socket (Linux)
Alloy.serve({
  unix: "\0my-abstract-socket",
  fetch(req) {
    return new Response("OK");
  },
});
```

---

## Code Generation (Codegen)

### Route Pattern Parsing & Translation

```ts
import { parseRoute, toRocketPath } from "@alloy-ts/server";

const routeDef = parseRoute({ method: "GET", path: "/users/:id?<age>&<name..>" });

const rocketPattern = toRocketPath("/users/:id/*");
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

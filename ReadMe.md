# Server

Node.js (TypeScript) NAPI bindings for the Rocket web framework and code generation utilities.

## Features

- **High-performance HTTP server (`Alloy.serve` / `serve`)**: Bun/Alloy-style routing backed by Rocket.
- **Rocket Configuration (`Config`)**: Full mirror of Rocket's server configuration (`Config.debugDefault()`, `Config.figment()`, `Config.tryFrom()`, limits, shutdown signals, log levels).
- **Rocket Server Lifecycle (`Rocket`)**: Direct programmatic control over mounting routes, registering catchers, ignition, and graceful shutdown.
- **TypeScript Code Generation (`codegen`)**:
  - `parseRoute` / `parseRoutes`: Extract route parameter definitions.
  - `toRocketPath`: Translate Express/Alloy/Bun route patterns (`/users/:id/*`) into Rocket syntax (`/users/<id>/<_..>`).
  - `generateModule` / `generateClient` / `generateTypes`: Generate strongly-typed client SDKs, route maps, and `uri()` helpers.

---

## Quick Start

### Basic Server (`Alloy.serve`)

```ts
import { serve, response, responseJson, responseRedirect, file } from "@alloy-ts/server";

const Alloy = {
  serve(options: any) {
    const { routes, fetch, port, hostname, unix } = options;
    return serve(routes, fetch, { port, hostname, unix });
  },
  file(path: string) {
    return file(path);
  },
};

const server = Alloy.serve({
  port: 3000,
  routes: {
    // Static routes
    "/api/status": response("OK"),

    // Dynamic routes
    "/users/:id": (req: any) => response(`Hello User ${req.params?.id}!`),

    // Per-HTTP method handlers
    "/api/posts": {
      GET: () => response("List posts"),
      POST: async (req: any) => responseJson({ created: true }),
    },

    // Wildcard fallback route
    "/api/*": responseJson({ message: "Not found" }, { status: 404 }),

    // Redirect
    "/blog/hello": responseRedirect("/blog/hello/world"),

    // Serve a file lazily
    "/favicon.ico": Alloy.file("./favicon.ico"),
  },

  // Fallback for unmatched routes
  fetch(req: any) {
    return response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);
```

---

## Rocket Configuration (`Config`)

Rocket's configuration system uses Figment to merge defaults, configuration files (`Rocket.toml`), and environment variables (`ROCKET_*`).

### Using `Config`

```ts
import { Config, type ConfigInit } from "@alloy-ts/server";

// Seeded with Rocket's debug defaults
const debugConfig = Config.debugDefault();
console.log(debugConfig.port); // 8000
console.log(debugConfig.address); // "127.0.0.1"

// Extracted from Rocket's default figment (defaults + Rocket.toml + ROCKET_*)
const figmentConfig = Config.figment();

// Customize properties
const config = new Config();
config.port = 8080;
config.address = "0.0.0.0";
config.logLevel = "normal";
config.workers = 8;
config.limits = {
  limits: {
    json: "10MiB",
    form: "64KiB",
  },
};
config.shutdown = {
  ctrlc: true,
  grace: 5,
  mercy: 5,
};

// Or pass overrides via Config.tryFrom()
const customConfig = Config.tryFrom({
  port: 9090,
  address: "0.0.0.0",
  logLevel: "critical",
} satisfies ConfigInit);
```

---

## Code Generation (`codegen`)

### Route Pattern Parsing & Translation

```ts
import { parseRoute, toRocketPath } from "@alloy-ts/server";

const routeDef = parseRoute({ method: "GET", path: "/posts/:slug?<query>&<rest..>" });
// routeDef.rocketPath -> "/posts/<slug>"
// routeDef.params -> [{ name: "slug", tsName: "slug", tsType: "string | number", ... }]
// routeDef.query -> [{ name: "query", ... }, { name: "rest", ... }]

const rocketPath = toRocketPath("/users/:id/*");
// "/users/<id>/<_..>"
```

### TypeScript Client SDK & Route Types Generation

```ts
import { parseRoutes, generateModule } from "@alloy-ts/server";

const routes = parseRoutes([
  { name: "getUser", method: "GET", path: "/users/:id", response: "User" },
  { name: "createPost", method: "POST", path: "/posts", body: "CreatePost", response: "Post" },
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

- Code check:

```bash
npm run check
```

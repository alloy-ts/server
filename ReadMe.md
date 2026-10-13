# Server

Node.js (TypeScript) NAPI bindings for the Rocket web framework and code generation utilities.

## Features

- **High-performance HTTP server (`Alloy.serve` / `serve`)**: Bun/Alloy-style routing backed by Rocket.
- **Rocket Server Lifecycle (`Rocket`)**: Direct programmatic control over mounting routes, registering catchers, ignition, and graceful shutdown.
- **Rocket Configuration (`Config`)**: Full mirror of Rocket's server configuration (`Config.debugDefault()`, `Config.releaseDefault()`, `Config.figment()`, `Config.tryFrom()`, limits, shutdown signals, log levels).
- **TypeScript Code Generation (`codegen`)**:
  - `parseRoute` / `parseRoutes`: Extract dynamic path/query parameters from route strings.
  - `toRocketPath`: Translate route syntax (`/users/:id/*`) into Rocket route syntax (`/users/<id>/<_..>`).
  - `generateClient` / `generateTypes` / `generateModule`: Generate strongly-typed fetch client SDKs and route map interfaces.

---

## Quick Start

### Basic Server (`Alloy.serve`)

```ts
import { Alloy, Response, file } from "@alloy-ts/server";

const server = Alloy.serve({
  port: 3000,
  routes: {
    // Static routes
    "/api/status": new Response("OK"),

    // Dynamic routes
    "/users/:id": (req) => new Response(`Hello User ${req.params?.id}!`),

    // Per-HTTP method handlers
    "/api/posts": {
      GET: () => new Response("List posts"),
      POST: async (req) => {
        return Response.json({ created: true });
      },
    },

    // Wildcard route
    "/api/*": Response.json({ message: "Not found" }, { status: 404 }),

    // Redirect
    "/blog/hello": Response.redirect("/blog/hello/world"),

    // Lazy file serving
    "/favicon.ico": file("./favicon.ico"),
  },

  // Fallback for unmatched routes
  fetch(req) {
    return new Response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);
```

---

## Rocket Configuration (`Config`)

Rocket's configuration system is based on Figment providers. The NAPI `Config` class exposes every documented Rocket server configuration property and provider helper to Node.js and TypeScript.

### Default Profiles & Factories

```ts
import { Config } from "@alloy-ts/server";

// Debug defaults (127.0.0.1:8000, profile: "debug", logLevel: "normal")
const debugConfig = Config.debugDefault();

// Release defaults (profile: "release", logLevel: "critical")
const releaseConfig = Config.releaseDefault();

// Extracted from Rocket.toml and ROCKET_* environment variables via Figment
const figmentConfig = Config.figment();
```

### Configuration Overrides (`tryFrom` / `from`)

```ts
import { Config } from "@alloy-ts/server";

const config = Config.tryFrom({
  address: "0.0.0.0",
  port: 9000,
  workers: 16,
  maxBlocking: 512,
  ident: "MyServer/1.0",
  logLevel: "debug",
  cliColors: false,
  limits: {
    limits: {
      json: "10MiB",
      form: "64KiB",
    },
  },
  shutdown: {
    ctrlc: true,
    signals: ["term", "hup"],
    grace: 5,
    mercy: 5,
  },
});
```

### Getters and Setters

Every configuration field is exposed as a getter/setter on `Config`:

```ts
const config = new Config();

config.address = "192.168.1.10";
config.port = 8080;
config.workers = 8;
config.logLevel = "critical";
config.ident = null; // Disable server identity header

console.log(`Configured for ${config.address}:${config.port} (${config.profile})`);
```

---

## Code Generation (Codegen)

### Route Pattern Parsing & Translation

```ts
import { parseRoute, toRocketPath } from "@alloy-ts/server";

const route = parseRoute({ method: "GET", path: "/users/:id?<age>&<name..>" });

const rocketPath = toRocketPath("/users/:id/*");
// "/users/<id>/<_..>"
```

### TypeScript Client SDK Generation

```ts
import { generateClient, parseRoutes } from "@alloy-ts/server";

const routes = parseRoutes([{ method: "GET", path: "/users/:id", response: "User" }]);

const clientCode = generateClient(routes, { clientName: "UserApiClient" });
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
npm run check
```

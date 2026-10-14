# Server

Node.js (TypeScript) NAPI bindings for the Rocket web framework and code generation utilities.

## Features

- **High-performance HTTP server (`Alloy.serve` / `serve`)**: Bun/Alloy-style declarative routing backed by Rocket.
- **Fluent App Builder (`App`)**: Rocket-style builder pattern (`new App()`, `app.get()`, `app.post()`, `app.launch()`).
- **Rocket Server Lifecycle (`Rocket`)**: Programmatic control over mounting routes, registering catchers, ignition, and graceful shutdown.
- **Rocket Configuration (`Config`)**: Full mirror of Rocket's server configuration (`Config.debugDefault()`, `Config.tryFrom()`, limits, shutdown signals, log levels).
- **TypeScript Code Generation (`codegen`)**:
  - `parseRoute` / `parseRoutes`: Extract dynamic path/query parameters and resolve route tables.
  - `toRocketPath`: Translate Alloy/Bun route syntax (`/users/:id/*`) into Rocket route syntax (`/users/<id>/<_..>`).
  - `generateTypes`: Generate TypeScript `RouteTypes` interfaces and context types for route tables.
  - `generateClient`: Generate a strongly-typed fetch client SDK for route tables with URL parameter builders.
  - `generateModule`: Generate a complete module with constants, types, `uri()` helper, and client class.

---

## Quick Start

### Basic Server (`Alloy.serve`)

```ts
import { Alloy, file, response, responseJson, responseRedirect } from "@alloy-ts/server";

const server = Alloy.serve({
  port: Number(process.env["PORT"] ?? 3000),
  hostname: "127.0.0.1",
  routes: {
    "/api/status": response("OK"),
    "/users/:id": (req) => response(`Hello User ${req.params["id"]}!`),
    "/api/posts": {
      GET: () => response("List posts"),
      POST: async (req) => responseJson({ created: true }),
    },
    "/blog/hello": responseRedirect("/blog/hello/world"),
    "/favicon.ico": file("./favicon.ico"),
  },
  fetch(req) {
    return response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);
```

### Fluent Builder (`App`)

```ts
import { App, response } from "@alloy-ts/server";

const app = new App();

app.get("/", async () => response("Hello, world!"));

app.launch();

console.log("Server running at http://localhost:3000");
```

---

## Rocket Configuration (`Config`)

```ts
import { Config } from "@alloy-ts/server";

const config = Config.tryFrom({
  port: 8080,
  address: "127.0.0.1",
  workers: 4,
  maxBlocking: 256,
  ident: "AlloyRocketServer",
  logLevel: "normal",
  limits: {
    limits: {
      json: "10MiB",
      form: "64KiB",
    },
  },
});
```

---

## Code Generation (Codegen)

### Route Pattern Parsing & Translation

```ts
import { parseRoutes, toRocketPath } from "@alloy-ts/server";

const routes = parseRoutes([
  { method: "GET", path: "/users/:id", response: "User" },
  { method: "POST", path: "/users", body: "CreateUser", response: "User" },
]);

const rocketPath = toRocketPath("/users/:id/*");
// "/users/<id>/<_..>"
```

### TypeScript Client SDK Generation

```ts
import { generateClient, generateModule } from "@alloy-ts/server";

const clientCode = generateClient(routes, { clientName: "UserApiClient" });
const fullModule = generateModule(routes, { baseUrl: "http://127.0.0.1:3000" });
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

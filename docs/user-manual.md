# `@alloy-ts/server` User Manual

`@alloy-ts/server` provides high-performance Node.js (TypeScript) NAPI bindings for the Rocket web framework alongside comprehensive TypeScript code generation utilities.

---

## Table of Contents

- [Quick Start](#quick-start)
- [HTTP Server (`Alloy.serve`)](#http-server-alloyserve)
- [Rocket Lifecycle (`Rocket`)](#rocket-lifecycle-rocket)
- [Configuration (`Config`)](#configuration-config)
- [Code Generation (`codegen`)](#code-generation-codegen)
  - [Route Pattern Parsing](#route-pattern-parsing)
  - [Pattern Translation](#pattern-translation)
  - [Client SDK Generation](#client-sdk-generation)
  - [Route Handler Generation](#route-handler-generation)
  - [TypeScript Interface Generation](#typescript-interface-generation)

---

## Quick Start

Install dependencies:

```bash
npm install @alloy-ts/server
```

Create a server (`index.ts`):

```ts
import { serve, response, responseJson } from "@alloy-ts/server";

const server = serve({
  routes: {
    "/": response("Hello from Rocket!"),
    "/api/status": responseJson({ status: "OK" }),
    "/users/:id": (req) => response(`User ID: ${req.params?.id}`),
  },
});

console.log(`Server listening on ${server.url}`);
```

---

## HTTP Server (`Alloy.serve`)

`serve` provides Bun/Alloy-style server ergonomics backed by Rocket's HTTP engine.

### Usage

```ts
import { serve, response, responseJson, responseRedirect, file } from "@alloy-ts/server";

const server = serve(
  {
    routes: {
      // Plain response
      "/": response("Welcome"),

      // Dynamic path parameters
      "/users/:id": (req) => response(`User: ${req.params.id}`),

      // Per-HTTP method routing
      "/api/posts": {
        GET: () => responseJson([{ id: 1, title: "First Post" }]),
        POST: async (req) => responseJson({ created: true }),
      },

      // Static file serving
      "/favicon.ico": file("./favicon.ico"),

      // Redirects
      "/old-path": responseRedirect("/new-path"),
    },
    fetch(req) {
      return response("Not Found", { status: 404 });
    },
  },
  undefined,
  { port: 3000, hostname: "0.0.0.0" },
);
```

To stop the server gracefully:

```ts
server.stop();
```

---

## Rocket Lifecycle (`Rocket`)

For direct programmatic control over Rocket's lifecycle (mounting routes, registering catchers, ignition, launch, and shutdown):

```ts
import { Rocket, response, responseJson } from "@alloy-ts/server";

const rocket = new Rocket({
  port: 8000,
  logLevel: "normal",
});

// Mount routes
rocket.mount("/", "GET", "/hello", async (req) => {
  return responseJson({ message: "Hello World" });
});

// Register custom status catchers (e.g. 404)
rocket.register("/", 404, async (req) => {
  return responseJson({ error: "Custom 404 Not Found" }, { status: 404 });
});

// Launch server
await rocket.launch();
```

To initiate graceful shutdown programmatically:

```ts
rocket.shutdown();
```

---

## Configuration (`Config`)

`Config` mirrors Rocket's configuration system (`rocket::Config`), populated from defaults, `Rocket.toml`, or environment variables (`ROCKET_*`).

### Debug Defaults & Partial Overrides

```ts
import { Config } from "@alloy-ts/server";

// Seeded with Rocket's debug defaults (port 8000, address 127.0.0.1, etc.)
const config = Config.debugDefault();
config.port = 3000;
config.address = "0.0.0.0";

// From partial overrides
const customConfig = Config.tryFrom({
  port: 8080,
  logLevel: "debug",
  limits: {
    limits: {
      json: "10MiB",
      form: "64KiB",
    },
  },
});
```

---

## Code Generation (`codegen`)

### Route Pattern Parsing

Extract dynamic path and query parameters from Rocket route strings:

```ts
import { parseRocketRoutePattern } from "@alloy-ts/server";

const params = parseRocketRoutePattern("/users/<id>?<age>&<name..>");
// Output:
// [
//   { name: "id", paramType: "string", isPath: true, isQuery: false, isOptional: false },
//   { name: "age", paramType: "string", isPath: false, isQuery: true, isOptional: false },
//   { name: "name", paramType: "string", isPath: false, isQuery: true, isOptional: true }
// ]
```

### Pattern Translation

Convert Rocket route syntax into Alloy/Bun route syntax:

```ts
import { translatePatternToAlloy } from "@alloy-ts/server";

const alloyPattern = translatePatternToAlloy("/users/<id>/<rest..>");
// Output: "/users/:id/*"
```

### Client SDK Generation

Generate a strongly-typed TypeScript `fetch` client class:

```ts
import { generateTypescriptClient } from "@alloy-ts/server";

const sdk = generateTypescriptClient(
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

console.log(sdk.code);
```

### Route Handler Generation

Generate route handler boilerplate:

```ts
import { generateRouteHandler } from "@alloy-ts/server";

const handler = generateRouteHandler({
  method: "POST",
  path: "/posts/<id>",
  name: "createPostHandler",
});

console.log(handler.code);
```

### TypeScript Interface Generation

Convert Rust field specifications into TypeScript interfaces:

```ts
import { generateTypescriptInterface } from "@alloy-ts/server";

const iface = generateTypescriptInterface("User", [
  { name: "id", type: "string" },
  { name: "email", type: "string" },
  { name: "age", type: "number", optional: true },
]);

console.log(iface.code);
```

# User Manual: Rocket NAPI Bindings for TypeScript & Node.js

`@alloy-ts/server` provides high-performance Node.js / TypeScript NAPI bindings for the Rust [Rocket](https://rocket.rs/) web framework.

The library exports JS and TS declarations directly from Rust NAPI macros (`#[napi]`) into `dist/index.js` and `dist/index.d.ts`. No hand-written TypeScript wrapper source files are required.

---

## 1. Quickstart

### Server Creation with `Alloy.serve` / `Bun.serve`

```ts
import { Alloy, response, responseJson, responseRedirect, file } from "@alloy-ts/server";

const server = Alloy.serve({
  port: 3000,
  hostname: "0.0.0.0",

  routes: {
    // Plain static response
    "/api/status": response("OK"),

    // Dynamic route with request parameters
    "/users/:id": (req) => {
      return response(`User ID: ${req.params.id}`);
    },

    // Per-HTTP method map
    "/api/posts": {
      GET: () => response("List posts"),
      POST: async (req) => {
        const body = req.body ? JSON.parse(Buffer.from(req.body).toString()) : {};
        return responseJson({ created: true, ...body });
      },
    },

    // Wildcard match
    "/api/*": responseJson({ message: "Not found" }, { status: 404 }),

    // Redirect
    "/blog/hello": responseRedirect("/blog/hello/world"),

    // Lazy static file
    "/favicon.ico": file("./favicon.ico"),
  },

  // Fallback catch-all handler
  fetch(_req) {
    return response("Not Found", { status: 404 });
  },
});

console.log(`Server listening at ${server.url}`);
```

---

## 2. Programmatic Server Builder (`App`)

The `App` class provides a chainable builder API:

```ts
import { App, response, responseJson } from "@alloy-ts/server";

const app = new App();

app
  .get("/hello", () => response("Hello!"))
  .post("/api/items", async (req) => {
    return responseJson({ id: 1 });
  });

const server = app.launch({ port: 8080 });
```

---

## 3. Rocket Configuration (`Config`)

Rocket configuration parameters can be inspected, created, or overridden:

```ts
import { Config } from "@alloy-ts/server";

// Inspect defaults
const debugConfig = Config.debugDefault();
console.log(debugConfig.address, debugConfig.port);

// Create custom overrides
const customConfig = Config.tryFrom({
  address: "0.0.0.0",
  port: 8080,
  workers: 8,
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

## 4. Route Metadata & Route URIs (`Route`, `RouteUri`)

Inspect and construct Rocket route metadata and route URIs:

```ts
import { Route, RouteUri } from "@alloy-ts/server";

// Route metadata and ranking
const route = Route.ranked(1, "POST", "/users/:id");
console.log(route.rank, route.method, route.uri);

// RouteUri base / origin / path parsing
const routeUri = new RouteUri("/api", "/users/<id>?<q>");
console.log(routeUri.base); // "/api"
console.log(routeUri.unmountedOrigin); // "/users/<id>?<q>"
console.log(routeUri.origin); // "/api/users/<id>?<q>"
console.log(routeUri.path()); // "/api/users/<id>"
console.log(routeUri.query()); // "<q>"
console.log(routeUri.defaultRank()); // -8
```

---

## 5. Custom Handlers & Outcomes (`Handler`, `outcomeSuccess`, `outcomeError`, `outcomeForward`)

Implement custom handler logic with Rocket request outcomes:

```ts
import { Handler, outcomeSuccess, outcomeError, outcomeForward, response } from "@alloy-ts/server";

const customHandler = new Handler(async (req) => {
  if (req.path === "/allowed") {
    return outcomeSuccess(response("Access granted"));
  }
  if (req.path === "/forbidden") {
    return outcomeError(403);
  }
  return outcomeForward(null);
});
```

---

## 6. TypeScript Codegen & URI Generation

Generate type definitions and strongly-typed client SDKs from route patterns:

```ts
import { parseRoutes, generateClient, generateTypes, generateModule } from "@alloy-ts/server";

const routes = parseRoutes([
  { method: "GET", path: "/users/:id", response: "User" },
  { method: "POST", path: "/users", body: "CreateUser", response: "User" },
]);

// Emit TypeScript route interfaces
const typesCode = generateTypes(routes);

// Emit strongly-typed fetch client
const clientCode = generateClient(routes, {
  clientName: "ApiClient",
  baseUrl: "http://localhost:3000",
});

// Emit complete standalone module
const moduleCode = generateModule(routes);
```

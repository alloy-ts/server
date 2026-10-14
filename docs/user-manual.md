# AlloyScript Rocket NAPI Bindings - User Manual

This manual explains how to run high-performance TypeScript web applications in Node.js powered by native Rust [Rocket](https://rocket.rs/) web framework NAPI bindings (`@alloy-ts/server`).

---

## Installation & Requirements

- **Node.js**: `v18.0.0` or higher (compatible with standard Node.js runtime).
- **TypeScript Runner**: We recommend running TypeScript files directly using `@oxc-node/core`:
  ```bash
  node --import @oxc-node/core server.ts
  ```

---

## 1. `Alloy.serve` API (Bun.serve compatible)

`Alloy.serve` provides a zero-boilerplate, single-object option configuration that mirrors the standard `Bun.serve` / `Alloy.serve` web server API.

### Basic Server Setup

```typescript
import { Alloy, Response } from "@alloy-ts/server";

const server = Alloy.serve({
  port: 3000, // Defaults to $BUN_PORT, $PORT, $NODE_PORT or 3000
  hostname: "127.0.0.1", // Defaults to "0.0.0.0"

  routes: {
    // 1. Static response
    "/api/status": new Response("OK"),

    // 2. Dynamic route parameters
    "/users/:id": (req) => {
      return new Response(`Hello User ${req.params.id}!`);
    },

    // 3. Per-HTTP method map
    "/api/posts": {
      GET: () => Response.json([{ id: 1, title: "First Post" }]),
      POST: async (req) => {
        const bodyStr = req.body ? new TextDecoder().decode(new Uint8Array(req.body)) : "{}";
        const body = JSON.parse(bodyStr);
        return Response.json({ created: true, ...body });
      },
    },

    // 4. Wildcard catch-all route
    "/api/*": Response.json({ message: "API route not found" }, { status: 404 }),

    // 5. Redirect route
    "/blog/hello": Response.redirect("/blog/hello/world"),

    // 6. Lazy static file loading
    "/favicon.ico": Alloy.file("./favicon.ico"),
  },

  // Fallback for unmatched routes
  fetch(req) {
    return new Response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);
```

---

## 2. Programmatic `App` Builder API

For developers who prefer Rocket's original builder pattern (`rocket::build()`), the `App` class provides a 1:1 fluent API counterpart in TypeScript.

```typescript
import { App, Response } from "@alloy-ts/server";

const app = new App();

app
  .get("/", () => new Response("Welcome home!"))
  .get("/users/:id", (req) => Response.json({ id: req.params.id }))
  .post("/api/items", async (req) => {
    return Response.json({ status: "created" });
  });

// Launch server on specified host/port
app.launch("127.0.0.1", 8000);
```

To stop the server programmatically:

```typescript
app.stop();
```

---

## 3. Rocket `Route`, `RouteUri`, and `Handler` Classes

Rocket routes requests based on collision precedence tables (ranks) and URIs. You can construct, inspect, and calculate ranks using the `Route`, `RouteUri`, and `Handler` classes:

```typescript
import { Route, RouteUri, Handler } from "@alloy-ts/server";

// 1. RouteUri inspection
const uri = RouteUri.parse("/base", "/foo/<bar>?a=1");
console.log(uri.base); // "/base"
console.log(uri.path); // "/base/foo/<bar>"
console.log(uri.query); // "a=1"
console.log(uri.asStr()); // "/base/foo/<bar>?a=1"

// 2. Create a Handler wrapper around a JS callback
const handler = new Handler((req: any) => new Response("hello " + req.url));
console.log(handler.isValid()); // true

// 3. Create a route with default computed rank (-9 for "GET /")
const index = new Route("GET", "/");
console.log(index.rank); // -9

// 4. Create a ranked route
const custom = Route.ranked("POST", "/foo?bar", 1);
console.log(custom.rank); // 1

// 5. Calculate Rocket's default rank for a URI pattern
console.log(Route.defaultRank("/?foo")); // -12 (static path, static query)
console.log(Route.defaultRank("/a/<b>")); // -5 (partial path, no query)
console.log(Route.defaultRank("/<b>/<c>")); // -1 (wild path, no query)
```

---

## 4. Server Configuration (`Config`)

Rocket's underlying configuration system (built on Figment) is fully exposed via the `Config` class.

### Creating Configuration

```typescript
import { Config } from "@alloy-ts/server";

// 1. Default configuration
const config = Config.debug(); // or Config.release()

// 2. Custom configuration builder
const customConfig = new Config({
  port: 8080,
  address: "127.0.0.1",
  workers: 8,
  maxBlocking: 256,
  ident: "MyAlloyServer",
  keepAlive: 10,
  cliColors: true,
  logLevel: "normal",
});

console.log(`Configured port: ${customConfig.port}`);
console.log(`Configured address: ${customConfig.address}`);
```

---

## 5. `Response` Utility Class

The native `Response` class supports Web-standard constructors and static helper methods:

```typescript
import { Response } from "@alloy-ts/server";

// Standard string / buffer response
const r1 = new Response("Hello World", {
  status: 200,
  headers: { "content-type": "text/plain" },
});

// JSON helper
const r2 = Response.json({ success: true, count: 42 }, { status: 200 });

// Redirect helper
const r3 = Response.redirect("https://example.com", 302);
```

---

## 6. Running Example Files

All examples in `examples/` can be run using Node.js with `@oxc-node/core`:

```bash
# Bun.serve / Alloy.serve server demo
node --import @oxc-node/core examples/server.ts

# Programmatic App routes demo
node --import @oxc-node/core examples/app-routes.ts

# RESTful API CRUD demo
node --import @oxc-node/core examples/rest-api.ts

# Authorization & Request Headers demo
node --import @oxc-node/core examples/auth-and-guards.ts

# Static files & HTML demo
node --import @oxc-node/core examples/static-and-files.ts

# Asynchronous I/O & DB simulation demo
node --import @oxc-node/core examples/async-operations.ts

# Rocket Config demo
node --import @oxc-node/core examples/config.ts
```

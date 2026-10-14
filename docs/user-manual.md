# `@alloy-ts/server` User Manual

The `@alloy-ts/server` package provides Node.js / TypeScript NAPI bindings for the **Rocket** web framework in Rust, giving you Bun/Alloy-compatible routing semantics and Rocket configuration controls directly in TypeScript.

---

## Installation

```bash
npm install @alloy-ts/server
```

---

## 1. High-Performance HTTP Server (`Alloy.serve`)

`Alloy.serve` (or `serve`) starts an HTTP server powered by Rocket's multi-threaded Toki runtime in a background thread.

```ts
import { Alloy, response, responseJson, responseRedirect, file } from "@alloy-ts/server";

const server = Alloy.serve({
  // Port defaults to $BUN_PORT, $PORT, $NODE_PORT or 3000. Use 0 for random dynamic port.
  port: 3000,
  hostname: "127.0.0.1",

  routes: {
    // Static response routes
    "/api/status": response("OK"),

    // Dynamic path segment routes (`:id`)
    "/users/:id": (req) => response(`Hello User ${req.params.id}!`),

    // Per-HTTP method handlers
    "/api/posts": {
      GET: () => response("List posts"),
      POST: async (req) => {
        const bodyText = req.body ? Buffer.from(req.body).toString("utf-8") : "";
        const json = bodyText ? JSON.parse(bodyText) : {};
        return responseJson({ created: true, ...json });
      },
    },

    // Wildcard route for unmatched `/api/*` paths
    "/api/*": responseJson({ message: "Not found" }, { status: 404 }),

    // Redirects
    "/blog/hello": responseRedirect("/blog/hello/world"),

    // Lazy file serving
    "/favicon.ico": file("./favicon.ico"),

    // HTML string response
    "/": "<h1>Welcome to Alloy!</h1>",
  },

  // Fallback for unmatched routes
  fetch(req) {
    return responseJson({ error: "Not Found", path: req.path }, { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);

// Stop gracefully
// server.stop();
```

---

## 2. Express / Rocket-Style Application (`App`)

The `App` class provides an imperative API mirroring `rocket::build().mount("/", routes![...])`.

```ts
import { App, response } from "@alloy-ts/server";

const app = new App();

app.get("/", async () => response("Hello, world!"));

app.post("/items", async (req) => {
  return response("Item created");
});

app.mount("/admin", "GET", "/dashboard", async () => {
  return response("Admin Dashboard");
});

const server = app.launch({ port: 8080 });
console.log(`App launched at ${server.url}`);
```

---

## 3. Rocket Configuration (`Config`)

Rocket's Figment configuration system is mirrored in the `Config` NAPI class.

```ts
import { Config } from "@alloy-ts/server";

// Seeded with debug defaults
const cfg = Config.debugDefault();
cfg.port = 8080;
cfg.address = "0.0.0.0";
cfg.workers = 16;
cfg.logLevel = "normal";

// Build from partial override object
const customCfg = Config.tryFrom({
  port: 9000,
  workers: 8,
  logLevel: "critical",
  shutdown: {
    ctrlc: true,
    grace: 5,
    mercy: 5,
  },
});

// Inspect Rocket environment figment
const figment = Config.figment();
console.log(`Figment port: ${figment.port}`);
```

---

## 4. Response Constructors

- `response(body?, init?)`: Plain body response.
- `responseJson(data, init?)`: JSON response with `Content-Type: application/json`.
- `responseRedirect(url, status?)`: Redirect response (default 302).
- `file(path)`: Static file marker for disk files.

---

## 5. TypeScript Code Generation (`codegen`)

Parse Rocket/Bun route strings and emit strongly-typed fetch SDK clients and route map interfaces.

```ts
import { parseRoutes, generateModule } from "@alloy-ts/server";

const routes = parseRoutes([
  { method: "GET", path: "/users/:id", response: "User" },
  { method: "POST", path: "/users", body: "CreateUser", response: "User" },
]);

const typescriptModuleCode = generateModule(routes, {
  baseUrl: "http://localhost:3000",
});
console.log(typescriptModuleCode);
```

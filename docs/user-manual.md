# @alloy-ts/server User Manual

`@alloy-ts/server` provides Node.js / TypeScript NAPI bindings for Rust's high-performance [Rocket web framework](https://rocket.rs/).

## Table of Contents

- [Installation](#installation)
- [Quickstart with `Alloy.serve`](#quickstart-with-alloyserve)
- [Rocket-style `App` Builder](#rocket-style-app-builder)
- [Configuration (`Config`)](#configuration-config)
- [Code Generation Utilities (`Codegen`)](#code-generation-utilities-codegen)
- [API Reference](#api-reference)

---

## Installation

Install `@alloy-ts/server` using your package manager:

```bash
npm install @alloy-ts/server
```

Build native bindings if developing from source:

```bash
npm run build:napi
```

---

## Quickstart with `Alloy.serve`

`Alloy.serve` starts a high-performance HTTP server backed by Rocket's multi-threaded Tokio runtime.

```ts
import { Alloy, response, responseJson, responseRedirect } from "@alloy-ts/server";

const server = Alloy.serve({
  port: 3000,
  hostname: "0.0.0.0",

  routes: {
    // Static text response
    "/api/status": response("OK"),

    // Dynamic path parameters
    "/users/:id": (req) => {
      return response(`Hello User ${req.params?.id}!`);
    },

    // Per-HTTP method map
    "/api/posts": {
      GET: () => responseJson([{ id: 1, title: "Rocket + TypeScript" }]),
      POST: async (req) => {
        return responseJson({ created: true, body: req.body }, { status: 201 });
      },
    },

    // Wildcard catch-all for prefix
    "/api/*": responseJson({ message: "Not found" }, { status: 404 }),

    // Redirect
    "/old-path": responseRedirect("/new-path"),

    // Lazy static file loading
    "/favicon.ico": Alloy.file("./favicon.ico"),
  },

  // Fallback handler
  fetch(_req) {
    return response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);
```

---

## Rocket-style `App` Builder

For developers familiar with Rocket's Rust API (`rocket::build().mount("/", routes![...])`), the `App` class provides a 1:1 runtime builder API:

```ts
import { App, response } from "@alloy-ts/server";

const app = new App({ port: 8080 });

app
  .get("/", async () => response("Hello from Rocket!"))
  .post("/items", async (req) => response("Item created", { status: 201 }));

const server = app.launch();
console.log(`Running on ${server.url}`);
```

---

## Configuration (`Config`)

Configure Rocket's runtime parameters programmatically using the `Config` NAPI class:

```ts
import { Config } from "@alloy-ts/server";

// Seeded with Rocket's debug defaults
const config = Config.debugDefault();
config.port = 8080;
config.address = "0.0.0.0";
config.workers = 8;
config.logLevel = "normal";

// Or create with partial overrides
const customConfig = Config.tryFrom({
  port: 9090,
  workers: 16,
  cliColors: true,
});
```

---

## Code Generation Utilities (`Codegen`)

The `Codegen` class and `parseRoutePattern` helper allow TypeScript tooling to parse Rocket route patterns and generate TypeScript route structures:

```ts
import { Codegen, parseRoutePattern } from "@alloy-ts/server";

// Parse Rocket route pattern parameters
const params = parseRoutePattern("/users/<id>/posts/<postId>?<sort>");
console.log(params.path_params); // ['id', 'postId']
console.log(params.query_params); // ['sort']

// Accumulate routes and generate TypeScript code
const codegen = new Codegen();
codegen.addRoute({
  method: "GET",
  path: "/users/<id>",
  name: "get_user",
  pathParams: ["id"],
  queryParams: [],
});

const output = codegen.generateTs();
console.log(output.tsCode);
```

---

## API Reference

### Response Helpers

- `response(body?, init?)`: Returns a standard Response object.
- `responseJson(data, init?)`: Returns a JSON Response object with `application/json` header.
- `responseRedirect(url, status?)`: Returns a redirect Response object (defaults to 302).
- `Alloy.file(path)` / `file(path)`: Specifies a file path to be served statically from disk.

### Server Lifecycle

- `server.url`: Server listening URL string (e.g. `http://0.0.0.0:3000`).
- `server.port`: Listening port number.
- `server.stop()`: Initiates graceful shutdown of the Rocket server.

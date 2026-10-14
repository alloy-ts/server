# @alloy-ts/server User Manual

Welcome to `@alloy-ts/server` — TypeScript and Node.js NAPI bindings for the [Rocket](https://rocket.rs/) web framework.

## Introduction

`@alloy-ts/server` allows Node.js and TypeScript developers to run a high-performance Rocket HTTP web server backed by Rust's multi-threaded Tokio runtime.

## Quickstart

### 1. Installation

```bash
npm install @alloy-ts/server
```

### 2. High-Performance Server (`Alloy.serve` / `serve`)

```ts
import { file, response, responseJson, serve } from "@alloy-ts/server";

const server = serve(
  {
    // Static text response
    "/": response("Hello from Rocket!"),

    // Dynamic path parameters (:id or <id>)
    "/users/:id": async (req) => {
      return responseJson({ id: req.params.id, name: "Alice" });
    },

    // Per-method handlers
    "/api/posts": {
      GET: async () => responseJson([{ id: 1, title: "First Post" }]),
      POST: async (req) => {
        const body = JSON.parse(Buffer.from(req.body).toString());
        return responseJson({ created: true, ...body });
      },
    },

    // Lazy static file serving
    "/favicon.ico": file("./favicon.ico"),
  },

  // Fallback fetch handler for unmatched routes
  async (_req) => response("Not Found", { status: 404 }),

  // Server options
  { port: 8080, hostname: "0.0.0.0" },
);

console.log(`Server listening at ${server.url}`);
```

### 3. Builder Pattern (`App`)

For developers translating canonical Rocket code (`rocket::build()`, `#[get]`, `#[launch]`):

```ts
import { App, response } from "@alloy-ts/server";

const app = new App({ port: 3000 });

app.get("/", async () => response("Hello World!"));
app.get("/users/:id", async (req) => response(`User ${req.params.id}`));

app.launch();
```

## Rocket Configuration (`Config`)

Rocket uses Figment for flexible profile-based configuration.

```ts
import { Config, Rocket } from "@alloy-ts/server";

// Seeded with debug defaults
const config = new Config();
config.port = 8080;
config.address = "127.0.0.1";
config.ident = "CustomRocket/1.0";
config.keepAlive = 10;
config.limits = { limits: { json: "2MiB", form: "64KiB" } };

// Or create overrides from partial object
const customConfig = Config.tryFrom({
  port: 9000,
  logLevel: "debug",
});
```

## Codegen & URI Generation (`uri`)

Replicates Rocket's `uri!` macro for URL-safe path parameter interpolation and query parameter encoding:

```ts
import { uri } from "@alloy-ts/server";

const userUri = uri("/users/:id", { id: "42" }); // "/users/42"
const searchUri = uri(
  "/search/:category",
  { category: "books" },
  { q: "rocket web framework", page: "1" },
); // "/search/books?q=rocket%20web%20framework&page=1"
```

## Examples

Run any included example using Node.js and `@oxc-node/core`:

```bash
node --import @oxc-node/core/register.mjs examples/server.ts
node --import @oxc-node/core/register.mjs examples/app-routes.ts
node --import @oxc-node/core/register.mjs examples/config.ts
node --import @oxc-node/core/register.mjs examples/uri-builder.ts
```

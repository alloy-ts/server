# Alloy Server User Manual (`@alloy-ts/server`)

`@alloy-ts/server` provides high-performance Node.js / TypeScript NAPI bindings for the [Rocket web framework](https://rocket.rs).

## Features

- **Bun / Alloy `serve` compatible API**: `Alloy.serve` and `Bun.serve` support routes, method maps, wildcards, redirects, and file markers.
- **`App` class**: Express-like syntax (`new App().get(...)`) re-expressing Rocket macros (`#[get]`, `routes!`, `#[launch]`) at runtime.
- **Rocket `Route` & `RouteUri` NAPI bindings**: Inspect and construct Rocket route metadata (`Route.ranked(rank, method, uri)`, `new RouteUri(base, uri)`).
- **Rocket `Handler` & `Outcome` NAPI bindings**: Represent route outcomes (`OutcomeHelper.success`, `OutcomeHelper.error`, `OutcomeHelper.forward`).
- **Rocket `Config` NAPI bindings**: Direct JS access to Rocket configuration options (address, port, workers, limits, shutdown, TLS, etc.).
- **TypeScript Codegen**: Generate typed client code and URI builders directly from route tables.

---

## Quickstart

### 1. Using `Alloy.serve` (Bun.serve compatible)

```ts
import { Alloy, Bun, response, responseJson } from "@alloy-ts/server";

const server = Alloy.serve({
  port: 3000,
  routes: {
    "/": response("Hello World!"),
    "/users/:id": (req) => response(`User: ${req.params.id}`),
    "/api/posts": {
      GET: () => responseJson({ posts: [] }),
      POST: async (req) => {
        const data = req.json();
        return responseJson({ created: true, ...data }, { status: 201 });
      },
    },
    "/favicon.ico": Bun.file("./favicon.ico"),
  },
  fetch(req) {
    return response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);
```

### 2. Using `App` Class

```ts
import { App, response } from "@alloy-ts/server";

const app = new App();

app.get("/", async () => response("Hello from Rocket!"));
app.get("/users/:id", async (req) => response(`User ID: ${req.params.id}`));

const server = app.launch({ port: 3000 });
console.log(`Server running at ${server.url}`);
```

### 3. Rocket `Route` and `RouteUri`

```ts
import { Route, RouteUri } from "@alloy-ts/server";

const route = Route.ranked(2, "GET", "/route/<path..>?query");
console.log(route.rank); // 2
console.log(route.method); // "GET"
console.log(route.uri); // "/route/<path..>?query"

const uri = new RouteUri("/base", "/foo/<bar>?a=1");
console.log(uri.base()); // "/base"
console.log(uri.path()); // "/base/foo/<bar>"
console.log(uri.query()); // "a=1"
```

### 4. Rocket Configuration

```ts
import { Config, Alloy, response } from "@alloy-ts/server";

const config = new Config();
config.port = 8080;
config.address = "127.0.0.1";

Alloy.serve({
  port: config.port,
  hostname: config.address,
  routes: {
    "/": response("Configured Rocket Server"),
  },
});
```

---

## Running TypeScript with `@oxc-node/core`

```bash
node --import @oxc-node/core/register examples/server.ts
```

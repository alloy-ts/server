# `@alloy-ts/server` User Manual

The `@alloy-ts/server` package provides high-performance Node.js / TypeScript NAPI bindings for the **Rocket** web framework, along with Bun/Alloy-compatible APIs, full server configuration controls, and route code generation utilities.

---

## Table of Contents

1. [Quick Start](#quick-start)
   - [Bun.serve / Alloy.serve Compatible API](#bunserve--alloyserve-compatible-api)
   - [Express / App Router Style API](#express--app-router-style-api)
2. [Server Options & Configuration](#server-options--configuration)
   - [Port & Hostname Resolution](#port--hostname-resolution)
   - [Rocket Config Class (`Config`)](#rocket-config-class-config)
3. [Request & Response Handling](#request--response-handling)
   - [Request Helper Methods (`req.json()`, `req.text()`, `req.params`)](#request-helper-methods)
   - [Response Helpers (`Response`, `Response.json()`, `Response.redirect()`)](#response-helpers)
   - [Static File Serving (`Bun.file` / `file`)](#static-file-serving)
4. [Low-Level Rocket Lifecycle (`Rocket`)](#low-level-rocket-lifecycle-rocket)
5. [TypeScript Code Generation (`codegen`)](#typescript-code-generation-codegen)
   - [Route Pattern Translation](#route-pattern-translation)
   - [Type-Safe Client SDK Generation](#type-safe-client-sdk-generation)

---

## Quick Start

### Bun.serve / Alloy.serve Compatible API

`Alloy.serve` (or `serve`) accepts an options object containing `routes`, optional `fetch` fallback, `port`, and `hostname`.

```ts
import { Alloy, Response, Bun } from "@alloy-ts/server";

const server = Alloy.serve({
  port: 3000,
  routes: {
    // Static response
    "/api/status": new Response("OK"),

    // Dynamic parameters (:id -> req.params.id)
    "/users/:id": (req) => new Response(`Hello User ${req.params.id}!`),

    // Per-HTTP method handlers
    "/api/posts": {
      GET: () => new Response("List posts"),
      POST: async (req) => {
        const body = await req.json();
        return Response.json({ created: true, ...body });
      },
    },

    // Wildcard route
    "/api/*": Response.json({ message: "Not found" }, { status: 404 }),

    // Redirect
    "/blog/hello": Response.redirect("/blog/hello/world"),

    // Static file serving
    "/favicon.ico": Bun.file("./favicon.ico"),
  },

  // Fallback handler for unmatched routes
  fetch(_req) {
    return new Response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);
```

### Express / App Router Style API

Translates Rocket's `#[get]`, `routes!`, and `#[launch]` into a TypeScript class structure:

```ts
import { Alloy, response } from "@alloy-ts/server";

export class App {
  private routes: Record<string, any> = {};

  get(path: string, handler: any) {
    this.routes[path] = { GET: handler };
  }

  post(path: string, handler: any) {
    this.routes[path] = { POST: handler };
  }

  launch(options?: any) {
    return Alloy.serve({
      routes: this.routes,
      ...options,
    });
  }
}

const app = new App();
app.get("/", async () => response("Hello, world!"));
app.launch({ port: 3000 });
```

---

## Server Options & Configuration

### Port & Hostname Resolution

Port configuration priority:

1. `port` passed to `serve({ port: ... })` (set `port: 0` for random port selection).
2. Environment variables: `BUN_PORT`, `PORT`, or `NODE_PORT`.
3. Default fallback: `3000`.

Read the bound port or URL from the returned `Server` instance:

```ts
console.log(server.port); // e.g. 3000 or randomly assigned port
console.log(server.url); // http://localhost:3000
```

### Rocket Config Class (`Config`)

You can directly construct, inspect, and mutate Rocket configurations in JS:

```ts
import { Config } from "@alloy-ts/server";

// Seeded with Rocket's debug defaults
const config = Config.debugDefault();
config.port = 8080;
config.logLevel = "normal";
config.workers = 8;
config.limits = { limits: { json: "10MiB", form: "64KiB" } };

// Or load defaults merged with Rocket.toml / ROCKET_* env variables:
const figmentConfig = Config.figment();
```

---

## Request & Response Handling

### Request Helper Methods

Route handlers receive a request context containing:

- `req.method`: Upper-case HTTP method (`GET`, `POST`, ...).
- `req.path`: Cleaned request path (`/users/123`).
- `req.query`: Query string (`?key=value`).
- `req.headers`: Header map.
- `req.params`: Extracted dynamic parameters (`req.params.id`).
- `await req.json()`: Helper method parsing request body as JSON.
- `await req.text()`: Helper method reading request body as text string.

### Response Helpers

- `new Response(body?, init?)`: Create standard response.
- `Response.json(data, init?)`: Create `application/json` response.
- `Response.redirect(url, status?)`: Create redirect response (default status `302`).
- `response(body?, init?)`: Free function helper for plain responses.
- `responseJson(data, init?)`: Free function helper for JSON responses.
- `responseRedirect(url, status?)`: Free function helper for redirects.

### Static File Serving

Use `Bun.file(path)` or `file(path)` to mark static file routes served directly by Rocket's async I/O:

```ts
import { Bun, file } from "@alloy-ts/server";

Alloy.serve({
  routes: {
    "/favicon.ico": Bun.file("./favicon.ico"),
    "/static/app.js": file("./dist/app.js"),
  },
});
```

---

## Low-Level Rocket Lifecycle (`Rocket`)

For granular control matching Rocket's Rust lifecycle:

```ts
import { Rocket } from "@alloy-ts/server";

const rocket = new Rocket({
  port: 8000,
  logLevel: "normal",
});

rocket.mount("/", "GET", "/hello", async (req) => {
  return { status: 200, headers: {}, body: Buffer.from("Hello from Rocket") };
});

await rocket.launch();
```

---

## TypeScript Code Generation (`codegen`)

### Route Pattern Translation

Translate Bun/Express path parameters into Rocket path specifications:

```ts
import { toRocketPath, parseRoute } from "@alloy-ts/server";

console.log(toRocketPath("/users/:id/*"));
// Outputs: "/users/<id>/<_..>"

const routeDef = parseRoute({ method: "GET", path: "/users/:id" });
console.log(routeDef.rocketPath); // "/users/<id>"
```

### Type-Safe Client SDK Generation

Generate a complete, strongly-typed `fetch` client SDK from your route definitions:

```ts
import { parseRoutes, generateClient } from "@alloy-ts/server";

const routes = parseRoutes([
  { method: "GET", path: "/users/:id", name: "getUser", response: "User" },
]);

const sdkCode = generateClient(routes, {
  clientName: "ApiClient",
  baseUrl: "http://localhost:3000",
});

console.log(sdkCode);
```

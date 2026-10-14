# User Manual: `@alloy-ts/server`

NAPI bindings bridging the [Rocket](https://rocket.rs) v0.5 web framework to TypeScript and Node.js.

---

## Table of Contents

1. [Overview](#overview)
2. [Installation](#installation)
3. [Quick Start](#quick-start)
4. [Core Server APIs](#core-server-apis)
   - [`Alloy.serve`](#alloyserve)
   - [`App` Builder Class](#app-builder-class)
5. [Request & Response](#request--response)
   - [`RequestInput`](#requestinput)
   - [`Response`](#response)
6. [Configuration (`Config`)](#configuration-config)
7. [Route Decorators](#route-decorators)
8. [Type-Safe URI Construction & Codegen](#type-safe-uri-construction--codegen)
   - [`uri()` and `buildRouteUri()`](#uri-and-buildrouteuri)
   - [`parseRoutePattern()`](#parseroutepattern)
   - [`generateRoutesTypeDefinitions()`](#generateroutestypedefinitions)
9. [Static Files](#static-files)

---

## Overview

`@alloy-ts/server` provides high-performance, native Node.js (NAPI) bindings for Rust's Rocket web framework. Rocket runs in a dedicated Tokio multi-threaded background runtime while exposing a Bun.serve-compatible API, class decorators, and codegen helpers in TypeScript.

---

## Installation

```bash
npm install @alloy-ts/server
```

---

## Quick Start

```typescript
import { Alloy, Response } from "@alloy-ts/server";

const server = Alloy.serve({
  port: 3000,
  routes: {
    "/": new Response("Hello, Rocket!"),
    "/users/:id": (req) => Response.json({ user: req.params.id }),
  },
});

console.log(`Server running at ${server.url}`);
```

---

## Core Server APIs

### `Alloy.serve`

Starts an HTTP server powered by Rocket.

```typescript
import { Alloy, file, Response } from "@alloy-ts/server";

const server = Alloy.serve({
  port: 8080, // Set port (or 0 for random port)
  hostname: "0.0.0.0", // Set host
  routes: {
    "/api/status": new Response("OK"),
    "/users/:id": (req) => new Response(`User ${req.params.id}`),
    "/api/posts": {
      GET: () => Response.json([{ id: 1 }]),
      POST: async (req) => {
        const body = req.body ? JSON.parse(Buffer.from(req.body).toString()) : {};
        return Response.json({ created: true, ...body }, { status: 201 });
      },
    },
    "/api/*": Response.json({ error: "Not Found" }, { status: 404 }),
    "/redirect": Response.redirect("/api/status"),
    "/favicon.ico": file("./favicon.ico"),
  },
  fetch(_req) {
    return new Response("Fallback Not Found", { status: 404 });
  },
});

// Stop server gracefully
server.stop();
```

### `App` Builder Class

1:1 translation of Rocket's `rocket::build()` builder pattern into TypeScript:

```typescript
import { App, response } from "@alloy-ts/server";

const app = new App();

app
  .get("/", () => response("Hello World"))
  .get("/users/:id", (req) => response(`User ${req.params.id}`))
  .post("/data", (req) => response("Saved", { status: 201 }));

const server = app.launch({ port: 3000 });
```

---

## Request & Response

### `RequestInput`

The object passed to TypeScript handlers for each incoming HTTP request:

```typescript
export interface RequestInput {
  method: string; // "GET", "POST", etc.
  path: string; // "/users/42"
  query: string; // "ref=abc"
  headers: HeaderInput[]; // [{ name: "host", value: "localhost" }]
  body?: Uint8Array | number[]; // Raw request body bytes
  params: Record<string, string>; // { id: "42" }
}
```

### `Response`

`Response` objects carry HTTP status code, headers, and body bytes:

```typescript
import { Response } from "@alloy-ts/server";

// Text response
const r1 = new Response("Plain text", { status: 200 });

// JSON response helper
const r2 = Response.json({ success: true }, { status: 200 });

// Redirect response helper
const r3 = Response.redirect("https://rocket.rs", 302);
```

---

## Configuration (`Config`)

Exposes Rocket's `Config` provider via NAPI:

```typescript
import { Config } from "@alloy-ts/server";

// Default debug config
const cfg = Config.debug_default();
console.log(cfg.port, cfg.address, cfg.workers);

// Custom override
const custom = Config.try_from({
  port: 8000,
  address: "127.0.0.1",
  workers: 8,
  keep_alive: 10,
});

// Mutate configuration
custom.port = 9000;
```

---

## Route Decorators

Class method decorators for route handler registration:

```typescript
import { Get, Post, Response } from "@alloy-ts/server";

class ApiController {
  @Get("/items")
  getItems() {
    return Response.json([{ id: 1, name: "Item 1" }]);
  }

  @Post("/items")
  createItem() {
    return new Response("Item Created", { status: 201 });
  }
}

new ApiController();
```

Available decorators: `@Get`, `@Post`, `@Put`, `@Delete`, `@Patch`, `@Head`, `@Options`, `@Route`, and `@Catch`.

---

## Type-Safe URI Construction & Codegen

### `uri()` and `buildRouteUri()`

Replaces `:param`, `<param>`, or `<param..>` in route patterns with encoded values:

```typescript
import { buildRouteUri, uri } from "@alloy-ts/server";

const path = uri("/users/:id", { id: 42 });
// => "/users/42"

const fullUrl = buildRouteUri("/search/<category>?<query>", {
  category: "books",
  query: "rocket",
});
// => "/search/books?query=rocket"
```

### `parseRoutePattern()`

Extracts path and query parameters from Rocket route syntax:

```typescript
import { parseRoutePattern } from "@alloy-ts/server";

const info = parseRoutePattern("/users/<id>/<action..>?<ref>");
// => { pathParams: [{ name: "id", isTrailing: false }, { name: "action", isTrailing: true }], queryParams: ["ref"] }
```

### `generateRoutesTypeDefinitions()`

Generates TypeScript interface definitions for registered routes:

```typescript
import { generateRoutesTypeDefinitions } from "@alloy-ts/server";

const tsCode = generateRoutesTypeDefinitions(["/api/users/<id>?<ref>"]);
// Generates interface ApiRoutes
```

---

## Static Files

Serve files lazily from disk:

```typescript
import { Alloy, file } from "@alloy-ts/server";

Alloy.serve({
  routes: {
    "/favicon.ico": file("./public/favicon.ico"),
  },
});
```

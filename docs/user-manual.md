# Alloy Server & Rocket NAPI User Manual

This manual explains how to use `@alloy-ts/server`, a high-performance Node.js (TypeScript) NAPI binding for the [Rocket web framework](https://rocket.rs/).

---

## Table of Contents

1. [Overview](#overview)
2. [Installation](#installation)
3. [Quick Start](#quick-start)
4. [HTTP Server (`Alloy.serve`)](#http-server-alloyserve)
   - [Route Definitions](#route-definitions)
   - [Per-Method Handlers](#per-method-handlers)
   - [Wildcard Routes](#wildcard-routes)
   - [Redirects & Static Files](#redirects--static-files)
   - [Fallback Handler](#fallback-handler)
   - [Port & Hostname Configuration](#port--hostname-configuration)
5. [Rocket Server Lifecycle (`Rocket`)](#rocket-server-lifecycle-rocket)
   - [Mounting Routes & Catchers](#mounting-routes--catchers)
   - [Custom Routes (`Route`) & Route URIs (`RouteUri`)](#custom-routes-route--route-uris-routeuri)
   - [Handler Outcomes (`HandlerBridge`)](#handler-outcomes-handlerbridge)
   - [Launch & Graceful Shutdown](#launch--graceful-shutdown)
6. [Rocket Configuration (`Config`)](#rocket-configuration-config)
   - [Inspecting Defaults](#inspecting-defaults)
   - [Partial Overrides (`Config.tryFrom`)](#partial-overrides-configtryfrom)
7. [TypeScript Code Generation (`codegen`)](#typescript-code-generation-codegen)
   - [Pattern Parsing (`parseRoute` / `parseRoutes`)](#pattern-parsing-parseroute--parseroutes)
   - [Pattern Translation (`toRocketPath`)](#pattern-translation-torocketpath)
   - [SDK & Type Generation (`generateModule`)](#sdk--type-generation-generatemodule)

---

## Overview

`@alloy-ts/server` provides dual paradigms for building web services backed by Rust's Rocket framework:

1. **Bun/Alloy-compatible High Level API (`serve`)**: Declarative route map syntax with low-overhead handler dispatching.
2. **Direct Rocket Control (`Rocket` / `Config` / `Route` / `RouteUri` / `HandlerBridge`)**: Direct control over Rocket's lifecycle, routes, ranks, catchers, and Figment configuration.

---

## Installation

```bash
npm install @alloy-ts/server
```

---

## Quick Start

```ts
import { serve, response } from "@alloy-ts/server";

const server = serve({
  routes: {
    "/": response("Hello from Rocket!"),
    "/users/:id": (req) => response(`User ID: ${req.params?.id}`),
  },
  port: 3000,
});

console.log(`Server running at ${server.url}`);
```

---

## HTTP Server (`Alloy.serve`)

### Route Definitions

`serve` accepts an object where keys are path patterns and values are handler functions, static responses, or per-method objects.

```ts
import { serve, response, responseJson } from "@alloy-ts/server";

serve({
  routes: {
    "/hello": response("Hello World"),
    "/api/data": () => responseJson({ ok: true }),
  },
});
```

### Per-Method Handlers

Group route handlers by HTTP method for a single path:

```ts
serve({
  routes: {
    "/api/posts": {
      GET: () => responseJson(["post 1", "post 2"]),
      POST: async (req) => {
        const body = req.body ? JSON.parse(new TextDecoder().decode(req.body)) : {};
        return responseJson({ created: true, ...body });
      },
    },
  },
});
```

### Wildcard Routes

Use `*` to match trailing multi-segment paths (translated to Rocket's `<_..>` catch-all):

```ts
serve({
  routes: {
    "/api/*": responseJson({ error: "API Endpoint Not Found" }, { status: 404 }),
  },
});
```

### Redirects & Static Files

```ts
import { serve, responseRedirect, file } from "@alloy-ts/server";

serve({
  routes: {
    "/old-path": responseRedirect("/new-path"),
    "/favicon.ico": file("./assets/favicon.ico"),
  },
});
```

### Fallback Handler

Catch unmatched routes with a top-level `fetch` callback:

```ts
serve(
  {
    "/": response("Home"),
  },
  (_req) => response("Not Found", { status: 404 }),
);
```

### Port & Hostname Configuration

If `port` is omitted, `serve` automatically inspects `BUN_PORT`, `PORT`, or `NODE_PORT` environment variables before defaulting to `3000`.

```ts
serve({ "/": response("OK") }, undefined, {
  port: 8080,
  hostname: "127.0.0.1",
});
```

---

## Rocket Server Lifecycle (`Rocket`)

### Mounting Routes & Catchers

```ts
import { Rocket, responseJson } from "@alloy-ts/server";

const rocket = new Rocket({
  port: 8000,
  address: "127.0.0.1",
});

rocket.register("/", 404, (_req) => {
  return responseJson({ error: "Custom 404 Catcher" }, { status: 404 });
});

rocket.mount("/", "GET", "/status", async () => {
  return responseJson({ status: "healthy" });
});

await rocket.launch();
```

### Custom Routes (`Route`) & Route URIs (`RouteUri`)

You can create explicit Rocket routes with custom ranks and inspect their RouteURIs:

```ts
import { Route } from "@alloy-ts/server";

const route = Route.ranked(1, "GET", "/api/items");
console.log(route.rank); // 1
console.log(route.uri); // "/api/items"

const rebased = route.mapBase("/v1");
console.log(rebased.uri); // "/v1/api/items"
```

### Handler Outcomes (`HandlerBridge`)

Express custom Rocket handler outcomes (`success`, `error`, `forward`):

```ts
import { HandlerBridge } from "@alloy-ts/server";

const handler = new HandlerBridge((req) => {
  if (!req.headers.find((h) => h.name.toLowerCase() === "authorization")) {
    return HandlerBridge.forward(401);
  }
  return HandlerBridge.success({
    status: 200,
    headers: [{ name: "content-type", value: "application/json" }],
    body: Buffer.from(JSON.stringify({ items: [] })),
  });
});
```

---

## Rocket Configuration (`Config`)

```ts
import { Config } from "@alloy-ts/server";

const config = Config.debugDefault();
console.log(config.port, config.logLevel);

const custom = Config.tryFrom({
  port: 9000,
  logLevel: "critical",
  shutdown: { grace: 10, ctrlc: true },
});
```

---

## TypeScript Code Generation (`codegen`)

```ts
import { parseRoutes, generateModule } from "@alloy-ts/server";

const routes = parseRoutes([
  { method: "GET", path: "/users/:id", response: "User" },
  { method: "POST", path: "/users", body: "CreateUser", response: "User" },
]);

const typescriptModuleCode = generateModule(routes, {
  clientName: "UserApiClient",
  baseUrl: "http://localhost:3000",
});

console.log(typescriptModuleCode);
```

# Rocket Web Framework for TypeScript / Node.js User Manual

## Overview

`@alloy-ts/server` brings Rust's Rocket web framework to Node.js and Bun runtime environments using high-performance NAPI-RS bindings. It provides seamless compatibility with Bun/Alloy server options (`Alloy.serve({ routes, fetch, port })`), Rocket configuration management via Figment, explicit route ranking (`Route`), custom request handler outcome structures (`HandlerOutcome`), and client SDK/type code generation.

---

## Key Features

- **Rocket Web Framework Engine**: High-throughput HTTP server powered by Rocket and Tokio async runtime in Rust.
- **Bun / Alloy Compatibility**: Drop-in `Alloy.serve` and `Bun.serve` APIs supporting route tables, static files (`Bun.file`), redirects, and fallback `fetch` handlers.
- **High-Level `App` Builder**: Fluid routing builder (`app.get()`, `app.post()`, `app.mount()`, `app.launch()`, `app.stop()`).
- **Rocket Configuration & Figment**: Native `Config` bindings for Rocket configuration files (`Rocket.toml`), environment overrides (`ROCKET_*`), and programmatic overrides (`Config.tryFrom`).
- **Rocket Routes & Ranking**: Explicit `Route` class wrapping Rocket's default ranking table computation and route creation.
- **Rocket Handlers & Outcomes**: `HandlerOutcome` and outcome helpers (`routeOutcomeSuccess`, `routeOutcomeError`, `routeOutcomeForward`) for Rocket request pipeline integration.
- **TypeScript Code Generation**: Automatically generate typed REST SDK clients, TypeScript route interfaces, and runtime `uri()` route builders from route definitions.

---

## Quickstart

### 1. Basic Server with `Alloy.serve`

```typescript
import { Alloy } from "@alloy-ts/server";

const server = Alloy.serve({
  port: 3000,
  routes: {
    "/": () => Alloy.response("Hello from Rocket + TypeScript!"),
    "/users/:id": (req: { params: { id: string } }) =>
      Alloy.Response.json({ userId: req.params.id }),
  },
});

console.log(`Server running at ${server.url}`);
```

### 2. Rocket-Style `App` Builder

```typescript
import { App } from "@alloy-ts/server";

const app = new App();

app
  .get("/", () => "Hello World!")
  .get("/api/v1/health", () => ({ status: "ok" }))
  .mount("/api/v1", {
    "/items/:id": (req: any) => ({ id: req.params.id }),
  });

const server = app.launch({ port: 8080 });
```

---

## Configuration Management (`Config`)

Rocket's `Config` class exposes Figment-backed configuration management:

```typescript
import { Config } from "@alloy-ts/server";

// Load configuration from default Figment (Rocket.toml + ROCKET_* env)
const config = Config.figment();
console.log(`Configured port: ${config.port}`);

// Create custom configuration overrides
const customConfig = Config.tryFrom({
  port: 8000,
  workers: 4,
  logLevel: "critical",
});

console.log(`Custom server running on port ${customConfig.port}`);
```

---

## Rocket Routes (`Route`)

The `Route` NAPI class wraps `rocket::Route`:

```typescript
import { Route } from "@alloy-ts/server";

// Create a route with Rocket's automatically computed default rank
const route = new Route("GET", "/users/<id>");
console.log(`Route method: ${route.method}, uri: ${route.uri}, rank: ${route.rank}`);

// Create a route with explicit rank and media type format
const customRoute = Route.create("getUser", "GET", "/users/<id>", 1, "application/json");
```

---

## Code Generation (`codegen`)

Generate strongly-typed TypeScript SDK clients and interface maps from route patterns:

```typescript
import { generateClient, generateTypes, parseRoutes } from "@alloy-ts/server";

const routes = parseRoutes([
  { method: "GET", path: "/users/:id", response: "User" },
  { method: "POST", path: "/users", body: "CreateUser", response: "User" },
]);

const typesDeclaration = generateTypes(routes);
const clientSDK = generateClient(routes);
```

---

## Examples

Run any of the provided TypeScript examples:

```bash
node --import @oxc-node/core/register examples/server.ts
node --import @oxc-node/core/register examples/app-routes.ts
node --import @oxc-node/core/register examples/config.ts
node --import @oxc-node/core/register examples/codegen.ts
```

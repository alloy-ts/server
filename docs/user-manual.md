# Rocket NAPI Bindings for TypeScript — User Manual

This manual explains how to use the Rocket web framework in TypeScript and Node.js through NAPI bindings.

---

## 1. Introduction

The `@alloy-ts/server` package bridges Rust's **Rocket web framework** to Node.js and TypeScript. It offers two primary ways to run HTTP servers:

1. **Bun/Alloy-compatible `Alloy.serve` API**: Simple, high-level route mapping and static file serving with Bun-like syntax.
2. **Rocket Lifecycle API (`Rocket`, `launch`, `rocketMain`)**: Direct access to Rocket's builder, configuration, custom route mounting, catchers, and graceful shutdown.

In addition, it provides a **Rocket Code Generation (`codegen`)** engine for route metadata, catcher declarations, dynamic type-safe URI building, and TypeScript route type generation.

---

## 2. Basic Setup (`Alloy.serve`)

```ts
import { Alloy, Response, file } from "@alloy-ts/server";

const server = Alloy.serve({
  port: 3000,
  hostname: "0.0.0.0",

  routes: {
    // Static response
    "/api/status": new Response("OK"),

    // Dynamic path parameters
    "/users/:id": (req) => {
      return new Response(`Hello User ${req.params?.id}!`);
    },

    // Per-HTTP method routing
    "/api/posts": {
      GET: () => Response.json({ posts: ["Rocket NAPI", "Alloy Server"] }),
      POST: async (req) => {
        const body = req.body ? JSON.parse(req.body.toString()) : {};
        return Response.json({ created: true, ...body }, { status: 201 });
      },
    },

    // Wildcard route matching
    "/api/*": Response.json({ message: "Not found" }, { status: 404 }),

    // Redirect
    "/blog/hello": Response.redirect("/blog/hello/world"),

    // Serve static files
    "/favicon.ico": file("./favicon.ico"),
  },

  // Fallback handler for unmatched routes
  fetch(req) {
    return new Response(`Not Found: ${req.path}`, { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);
```

---

## 3. Server Configuration (`Config`)

Rocket's configuration is managed via the `Config` class, mirroring `rocket::Config`:

```ts
import { Config, Rocket } from "@alloy-ts/server";

// Seed from debug or release defaults
const cfg = Config.debugDefault();
cfg.port = 8080;
cfg.address = "0.0.0.0";
cfg.workers = 8;
cfg.logLevel = "normal";
cfg.limits = {
  limits: {
    json: "10MiB",
    form: "64KiB",
  },
};

// Create a Rocket server using ConfigInit overrides
const server = new Rocket({
  port: 8080,
  address: "0.0.0.0",
  workers: 4,
  logLevel: "normal",
});
```

---

## 4. Rocket Code Generation (`codegen`)

### Route Definition Generation

```ts
import { codegen } from "@alloy-ts/server";

// Method-specific route definition
const getUser = codegen.get("/users/:id", { name: "get_user" });
console.log(getUser.rocketPath); // "/users/<id>"

// Batch route generation
const apiRoutes = codegen.routes([
  { method: "GET", uri: "/items" },
  { method: "POST", uri: "/items", data: "<item>" },
]);
```

### Type-Safe URI Generation

Generates URL-encoded URIs matching Rocket's `uri!` macro dynamic semantics:

```ts
import { codegen } from "@alloy-ts/server";

const userUri = codegen.uri("/users/:id", { id: "42", section: "profile" });
console.log(userUri.uri); // "/users/42?section=profile"
```

### TypeScript Route Type Emission

```ts
import { codegen } from "@alloy-ts/server";

const routes = codegen.routes([
  { method: "GET", uri: "/users/:id" },
  { method: "POST", uri: "/posts", data: "<post>" },
]);

const declarations = codegen.generateTsDeclarations(routes);
console.log(declarations);
```

---

## 5. Graceful Shutdown & Lifecycle Methods

```ts
// Stop Alloy.serve HTTP server
server.stop();

// Shutdown Rocket instance
app.shutdown();
```

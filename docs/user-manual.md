# User Manual: `@alloy-ts/server` (Rocket NAPI for TypeScript)

`@alloy-ts/server` provides native Node.js and TypeScript bindings for Rust's [Rocket](https://rocket.rs/) web framework, giving TypeScript applications access to Rocket's multi-threaded, high-performance HTTP runtime.

---

## 1. Quick Start

### Basic Server (`Alloy.serve`)

```ts
import { Alloy } from "@alloy-ts/server";

const server = Alloy.serve({
  port: 3000,
  routes: {
    // Static text response
    "/api/status": Alloy.Response("OK"),

    // Dynamic route with path parameter (:id)
    "/users/:id": (req) => {
      return Alloy.Response(`Hello User ${req.params.id}!`);
    },

    // Per-HTTP-method handlers
    "/api/posts": {
      GET: () => Alloy.Response("List posts"),
      POST: async (req) => {
        const body = req.body ? JSON.parse(Buffer.from(req.body).toString()) : {};
        return Alloy.Response.json({ created: true, ...body });
      },
    },

    // Wildcard route
    "/api/*": Alloy.Response.json({ message: "Not found" }, { status: 404 }),

    // HTTP Redirect
    "/blog/hello": Alloy.Response.redirect("/blog/hello/world"),

    // Serve static files lazily from disk
    "/favicon.ico": Alloy.file("./favicon.ico"),
  },

  // Fallback handler for unmatched routes
  fetch(req) {
    return Alloy.Response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);
```

---

## 2. Expressive Application Router (`App`)

The `App` router re-expresses Rocket's `rocket::build()` and `#[get]` attributes in fluent TypeScript:

```ts
import { App, response } from "@alloy-ts/server";

const app = new App({ port: 3000 });

app.get("/", async () => response("Hello, world!"));
app.get("/users/:id", async (req) => response(`User: ${req.params.id}`));
app.post("/items", async (req) => response("Item created", { status: 201 }));

const server = app.launch();
console.log(`Server listening at ${server.url}`);
```

---

## 3. Type-Safe URI Construction (`Alloy.uri`)

Construct URL-safe, percent-encoded URIs corresponding to Rocket's `uri!` macro:

```ts
import { Alloy } from "@alloy-ts/server";

const url = Alloy.uri("/users/:id/posts/:post_id", {
  id: 42,
  post_id: "hello world & rust",
});

// Output: /users/42/posts/hello%20world%20%26%20rust
console.log(url);
```

---

## 4. Rocket Configuration (`Config`)

Inspect and mutate Rocket's server configuration:

```ts
import { Config, Rocket } from "@alloy-ts/server";

// Default profiles
const debugConfig = Config.debugDefault();
const releaseConfig = Config.releaseDefault();

// Partial overrides
const customConfig = Config.tryFrom({
  port: 8080,
  address: "0.0.0.0",
  logLevel: "normal",
  workers: 8,
});

// Programmatically configure Rocket
const rocket = new Rocket({
  port: customConfig.port,
  address: customConfig.address,
});
```

---

## 5. Codegen Helpers

Utilities for compile-time route attribute parsing and TypeScript manifest generation:

```ts
import {
  parseRouteAttribute,
  parseCatcherAttribute,
  generateTsRouteManifest,
} from "@alloy-ts/server";

const routeMeta = parseRouteAttribute("get", "/api/v1/users", "list_users", 1, "json");
const catcherMeta = parseCatcherAttribute("404", "not_found_catcher");

const manifest = generateTsRouteManifest([routeMeta]);
console.log(manifest);
```

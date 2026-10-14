# Rocket Web Framework NAPI User Manual

`@alloy-ts/server` provides NAPI bindings bridging Rust's Rocket web framework to Node.js and TypeScript.

---

## 1. Quick Start with `Alloy.serve`

```ts
import { serve, response, responseJson, responseRedirect, file } from "@alloy-ts/server";

const server = serve({
  routes: {
    // Static text route
    "/api/status": response("OK"),

    // Dynamic parameter route (:id or <id>)
    "/users/:id": (req) => {
      return response(`Hello User ${req.params?.id}!`);
    },

    // Per-HTTP method map
    "/api/posts": {
      GET: () => responseJson([{ id: 1, title: "Hello Rocket" }]),
      POST: async (req) => {
        const body = req.body ? JSON.parse(Buffer.from(req.body).toString()) : {};
        return responseJson({ created: true, ...body }, { status: 201 });
      },
    },

    // Wildcard fallback
    "/api/*": responseJson({ message: "Not found" }, { status: 404 }),

    // Redirect
    "/blog/hello": responseRedirect("/blog/hello/world"),

    // Static file
    "/favicon.ico": file("./favicon.ico"),
  },

  fetch(_req) {
    return response("Not Found", { status: 404 });
  },

  port: 8080,
  hostname: "127.0.0.1",
});

console.log(`Server running at ${server.url}`);
```

---

## 2. Rocket Route & Ranking (`Route`)

```ts
import { Route, computeDefaultRank } from "@alloy-ts/server";

// Compute default rank based on URI specificity
const rank = computeDefaultRank("/foo/<id>?query"); // -8 (partial path, static query)

// Create a route instance
const route = new Route("GET", "/route/<path..>?query", "route_name", "json");
console.log(route.rank); // -8
console.log(route.method); // "GET"

// Explicitly ranked route
const customRoute = Route.ranked(2, "POST", "/submit");
console.log(customRoute.rank); // 2
```

---

## 3. Server Configuration (`Config`)

```ts
import { Config } from "@alloy-ts/server";

// Initialize with debug defaults
const config = new Config();
config.port = 8000;
config.address = "127.0.0.1";
config.workers = 16;

// Configure streaming read limits
config.limits = {
  limits: {
    form: "64KiB",
    json: "10MiB",
  },
};

// Partial override constructor
const customConfig = Config.tryFrom({
  port: 9000,
  logLevel: "normal",
  shutdown: { ctrlc: true, grace: 5 },
});
```

---

## 4. Route Decorators & Codegen Utilities

```ts
import { Get, Post, uri, routes, Codegen, responseJson } from "@alloy-ts/server";

class UserController {
  @Get("/users/<id>")
  getUser() {
    return responseJson({ id: 1, name: "Alice" });
  }

  @Post("/users")
  createUser() {
    return responseJson({ created: true });
  }
}

const controller = new UserController();
const activeRoutes = routes(controller.getUser, controller.createUser);

// Type-safe URI generation
const userUrl = uri("/users/<id>?<rank>", { id: 42, rank: "admin" }, "/api");
// Result: "/api/users/42?rank=admin"

// Parse route attribute metadata
const meta = Codegen.parseRoute("GET", "/person/<name>?<age>");
```

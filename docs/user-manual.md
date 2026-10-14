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

## 2. Rocket Route, RouteUri, Catcher & Outcomes (`Route`, `RouteUri`, `Catcher`, `HandlerOutcome`)

```ts
import {
  Route,
  RouteUri,
  Catcher,
  computeDefaultRank,
  createSuccessOutcome,
  response,
} from "@alloy-ts/server";

// Compute default rank based on URI specificity
const rank = computeDefaultRank("/foo/<id>?query"); // -8 (partial path, static query)

// Create a Route instance
const route = new Route("GET", "/route/<path..>?query", "route_name", "json");
console.log(route.rank); // -8
console.log(route.method); // "GET"

// RouteUri instance
const routeUri = new RouteUri("/base", "/foo/<bar>?a=1");
console.log(routeUri.base); // "/base"
console.log(routeUri.unmountedOrigin); // "/foo/<bar>?a=1"
console.log(routeUri.origin); // "/base/foo/<bar>?a=1"

// Catcher instance
const catcher = new Catcher(404, "not_found_catcher", "/api");
console.log(catcher.code); // 404

// Custom handler outcome
const outcome = createSuccessOutcome(response("Success"));
console.log(outcome.kind); // "success"
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

## 4. Codegen Utilities (`Codegen`)

```ts
import { buildUri, Codegen } from "@alloy-ts/server";

// Type-safe URI generation
const userUrl = buildUri("/users/<id>?<rank>", { id: "42", rank: "admin" }, "/api");
// Result: "/api/users/42?rank=admin"

// Parse route attribute metadata
const meta = Codegen.parseRoute("GET", "/person/<name>?<age>");
```

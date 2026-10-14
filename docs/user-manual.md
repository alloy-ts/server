# Rocket NAPI Bindings User Manual

`@alloy-ts/server` provides high-performance Node.js / TypeScript NAPI bindings for the Rust Rocket web framework.

---

## 1. Quickstart (`Alloy.serve`)

Start a high-performance HTTP server with Bun/Alloy-style routing backed by Rocket:

```ts
import { serve, response, responseJson, responseRedirect, file } from "@alloy-ts/server";

const server = serve({
  port: 3000,
  routes: {
    // Static response
    "/api/status": response("OK"),

    // Dynamic path parameters
    "/users/:id": (req) => response(`Hello User ${req.params?.id}!`),

    // Per-HTTP method map
    "/api/posts": {
      GET: () => response("List posts"),
      POST: async (req) => responseJson({ created: true }),
    },

    // Wildcard route
    "/api/*": responseJson({ message: "Not found" }, { status: 404 }),

    // Redirect
    "/blog/hello": responseRedirect("/blog/hello/world"),

    // Lazy file serving
    "/favicon.ico": file("./favicon.ico"),
  },

  // Fallback handler for unmatched paths
  fetch(req) {
    return response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);
```

---

## 2. Express/Rocket App Builder (`App`)

For programmatic route registration mirroring Rocket's `rocket::build()` and `#[get]` routes:

```ts
import { App, response } from "@alloy-ts/server";

const app = new App();

app.get("/", async () => response("Hello, world!"));
app.get("/users/:id", async (req) => response(`User ${req.params.id}`));

const server = app.launch({ port: 3000 });
```

---

## 3. Rocket Configuration (`Config`)

Rocket's configuration system uses Figment providers under the hood.

### Creating Configuration Instances

```ts
import { Config } from "@alloy-ts/server";

// Debug defaults (127.0.0.1:8000, profile: "debug", logLevel: "normal")
const debugCfg = Config.debugDefault();

// Release defaults (profile: "release", logLevel: "critical")
const releaseCfg = Config.releaseDefault();

// Extracted from Rocket.toml and ROCKET_* environment variables
const figmentCfg = Config.figment();

// Partial overrides onto debug defaults
const customCfg = Config.tryFrom({
  address: "0.0.0.0",
  port: 9000,
  workers: 8,
  logLevel: "debug",
  limits: {
    limits: {
      json: "10MiB",
      form: "64KiB",
    },
  },
});
```

### Getters and Setters

Every Rocket configuration parameter is accessible via getters and setters:

```ts
const cfg = new Config();

cfg.address = "127.0.0.1";
cfg.port = 8080;
cfg.workers = 16;
cfg.logLevel = "normal";
cfg.ident = null; // Disable Server response header
```

---

## 4. Route, RouteUri, and Handler Bindings

### Route (`Route`)

Exposes Rocket's `Route` metadata and rank calculation:

```ts
import { Route } from "@alloy-ts/server";

const route = new Route("GET", "/api/status");
console.log(route.rank); // Default computed rank (-9)

const ranked = Route.ranked(2, "POST", "/submit");
console.log(ranked.rank); // Explicit rank (2)
```

### RouteUri (`RouteUri`)

Exposes parsed base mount point, path, query, and default rank metadata:

```ts
import { RouteUri } from "@alloy-ts/server";

const uri = new RouteUri("/base", "/foo/bar?a=1");
console.log(uri.base); // "/base"
console.log(uri.path); // "/base/foo/bar"
console.log(uri.query); // "a=1"
```

### Handler Outcomes

Produce typed outcome results for custom handler procedures:

```ts
import { outcomeSuccess, outcomeError, outcomeForward } from "@alloy-ts/server";

const ok = outcomeSuccess("OK payload");
const err = outcomeError(400);
const fwd = outcomeForward(404);
```

---

## 5. TypeScript Code Generation (`codegen`)

Extract route definitions and emit client SDKs:

```ts
import { parseRoutes, generateModule } from "@alloy-ts/server";

const defs = parseRoutes([{ method: "GET", path: "/users/:id", response: "User" }]);

const sdkCode = generateModule(defs);
```

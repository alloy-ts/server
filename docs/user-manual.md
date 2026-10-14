# `@alloy-ts/server` User Manual

The `@alloy-ts/server` library provides high-performance NAPI-RS bindings for the **Rocket** web framework in TypeScript and Node.js.

---

## 1. Quick Start

### Installation

Ensure native bindings are built for your platform:

```bash
npm install @alloy-ts/server
```

### Running Examples with `@oxc-node/core`

You can run TypeScript examples directly using Node.js and `@oxc-node/core`:

```bash
node --import @oxc-node/core/register examples/server.ts
node --import @oxc-node/core/register examples/app-routes.ts
```

---

## 2. Server API (`Alloy.serve`)

`Alloy.serve` starts a Rocket HTTP server on a Tokio background runtime.

```ts
import { Alloy, Response } from "@alloy-ts/server";

const server = Alloy.serve({
  port: 8080,
  hostname: "127.0.0.1",

  routes: {
    // Static response
    "/api/status": new Response("OK"),

    // Dynamic path parameters (:id)
    "/users/:id": (req) => new Response(`User ${req.params?.id}`),

    // Method map
    "/api/posts": {
      GET: () => Response.json([{ id: 1, title: "Hello" }]),
      POST: async (req) => {
        const body = req.body ? JSON.parse(req.body.toString()) : {};
        return Response.json({ created: true, ...body }, { status: 201 });
      },
    },

    // Wildcard route
    "/api/*": Response.json({ message: "Not found" }, { status: 404 }),

    // Redirect
    "/blog/hello": Response.redirect("/blog/hello/world"),

    // File serving
    "/favicon.ico": Alloy.file("./favicon.ico"),
  },

  // Fallback handler
  fetch(_req) {
    return new Response("Not Found", { status: 404 });
  },
});

console.log(`Listening on ${server.url}`);
```

### Environment Variable Fallback

If `port` is omitted, `Alloy.serve` checks `BUN_PORT`, `PORT`, and `NODE_PORT` environment variables before defaulting to `3000`.

---

## 3. Rocket Macro counterpart (`App`)

The `App` class provides a 1:1 builder API corresponding to Rocket's procedural macros (`rocket::build()`, `#[get]`, `routes!`, `#[launch]`):

```ts
import { App, response } from "@alloy-ts/server";

const app = new App();

app.get("/", async () => response("Hello, world!"));
app.post("/items", async (req) => response({ ok: true }, 201));

await app.launch();
```

---

## 4. Route Metadata (`Route`) & Catchers (`Catcher`)

Inspect and construct route and catcher metadata:

```ts
import { Route, Catcher, createFairingInfo } from "@alloy-ts/server";

// Default computed rank (-9 for static path with no query)
const route = new Route("GET", "/foo/bar", "index", "json");
console.log(route.rank); // -9

// Custom rank override
const customRoute = Route.ranked(2, "POST", "/users", "create_user");
console.log(customRoute.rank); // 2

// Fairing metadata
const fairing = createFairingInfo("Request Timer", {
  ignite: false,
  liftoff: false,
  request: true,
  response: true,
  shutdown: false,
  singleton: false,
});
console.log(fairing.name); // "Request Timer"
```

---

## 5. Rocket Configuration (`Config`)

Customize Rocket configuration backed by Figment:

```ts
import { Config, Limits, Shutdown } from "@alloy-ts/server";

const config = Config.debugDefault();
config.port = 8000;
config.address = "0.0.0.0";
config.ident = "Rocket-TS";

config.limits = new Limits({
  json: 10 * 1024 * 1024,
});

config.shutdown = new Shutdown({
  ctrlc: true,
  grace: 5,
  mercy: 2,
});
```

---

## 6. Codegen Helpers (`generateRoutesManifest`, `buildUri`)

Construct type-safe URIs and route manifests:

```ts
import { generateRoutesManifest, buildUri } from "@alloy-ts/server";

const uri = buildUri("/users/<id>", { id: "42" });
console.log(uri); // "/users/42"

const manifest = generateRoutesManifest([
  { method: "GET", path: "/users/<id>", uri: "/users/<id>", name: "get_user" },
]);
console.log(manifest);
```

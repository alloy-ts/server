# `@alloy-ts/server`

NAPI-RS Node.js & TypeScript bindings for the [Rocket](https://rocket.rs/) web framework.

`@alloy-ts/server` bridges Rocket's high-performance Rust web runtime to Node.js and TypeScript applications using native NAPI-RS bindings. It provides Bun/Alloy-compatible server APIs, route handling, configuration management via Figment, error catchers, type-safe route URI generation, decorator attributes, and code generation utilities.

---

## Features

- **Rocket Server Runtime**: Launch high-performance Rocket HTTP servers powered by Tokio and NAPI threadsafe functions.
- **Express / Rocket Route Matching**: Native support for route parameters (`/users/:id` $\rightarrow$ `/users/<id>`), method maps (`GET`, `POST`, `PUT`, `DELETE`), static files, redirects, and wildcards (`/api/*`).
- **Figment Configuration (`Config`)**: Full Rocket profile configuration (`debugDefault`, `releaseDefault`, `tryFrom`) with support for port, address, workers, max blocking threads, keep-alive, limits, shutdown grace periods, log levels, secret keys, and TLS.
- **Error Catchers (`Catcher`)**: Register custom status catchers (`404`, `500`) or default error catchers with base URI mapping (`map_base`).
- **Codegen & Route Utilities**:
  - `parseRoutePattern`: Translates TS route patterns to Rocket syntax and extracts parameter names.
  - `buildRouteUri` / `uri`: Type-safe parameter interpolation and URL encoding.
  - `parseCatcherCode`: Flexible parsing for numeric and default catcher status codes.
  - `generateRouteManifest`: Compiles route metadata tables.
- **High-Level `App` & Decorators**: `new App()`, `@get`, `@post`, `@put`, `@del`, `@route`, and `@catchCatcher`.

---

## Quickstart Example

```ts
import Alloy, { Response } from "@alloy-ts/server";

const server = Alloy.serve({
  port: 3000,
  routes: {
    // Plain text response
    "/": "Hello from Rocket web framework in TypeScript!",

    // Route parameters
    "/users/:id": (req) => {
      return new Response(`User ID: ${req.params.id}`);
    },

    // Method handlers
    "/api/posts": {
      GET: () => Response.json([{ id: 1, title: "Rocket Web Framework" }]),
      POST: async (req) => {
        const body = req.body ? JSON.parse(req.body.toString()) : {};
        return Response.json({ created: true, ...body }, { status: 201 });
      },
    },

    // Static file serving
    "/favicon.ico": Alloy.file("./public/favicon.ico"),

    // Redirects
    "/old-link": Response.redirect("/new-link"),
  },

  fetch(_req) {
    return new Response("Not Found", { status: 404 });
  },
});

console.log(`Rocket server running at ${server.url}`);
```

---

## Class-Based App API (`rocket::build()`)

```ts
import { App, response } from "@alloy-ts/server";

const app = new App({ port: 8080 });

app.get("/", async () => response("Hello, world!"));
app.get("/users/:id", async (req) => response(`User: ${req.params.id}`));

app.launch();
console.log("Server launched at http://localhost:8080");
```

---

## Decorators Example

```ts
import { get, post, catchCatcher, launch, Response } from "@alloy-ts/server";

class UserController {
  @get("/users")
  listUsers() {
    return Response.json([{ id: "1", name: "Alice" }]);
  }

  @post("/users")
  createUser() {
    return Response.json({ created: true }, { status: 201 });
  }

  @catchCatcher(404)
  notFound() {
    return new Response("Not Found", { status: 404 });
  }
}

new UserController();
launch();
```

---

## Configuration (`Config`)

```ts
import { Config, serve, Response } from "@alloy-ts/server";

const config = Config.tryFrom({
  port: 8080,
  address: "127.0.0.1",
  workers: 8,
  logLevel: "normal",
  shutdown: {
    grace: 5,
    mercy: 5,
    ctrlc: true,
  },
});

const server = serve({
  port: config.port,
  hostname: config.address,
  fetch() {
    return new Response("Configured Rocket Server");
  },
});
```

---

## Type-Safe URI Generation (`uri!`)

```ts
import { uri, parseRoutePattern } from "@alloy-ts/server";

// Inspect route patterns
const info = parseRoutePattern("/users/:id/posts/:postId");
console.log(info.rocketPattern); // "/users/<id>/posts/<postId>"

// Construct URL-safe route URIs
const userUri = uri("/users/:id/posts/:postId", { id: "42", postId: "101" });
console.log(userUri); // "/users/42/posts/101"
```

---

## Development & Building

### Build Native NAPI-RS Bindings

```bash
npm run build
```

### Run Unit Tests

```bash
npm test
```

### Type Check & Code Formatting

```bash
npm run check
```

---

## License

MIT

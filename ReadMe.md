# @alloy-ts/server

Node.js (TypeScript) NAPI bindings for the Rocket web framework and code generation utilities.

## Features

- **High-performance HTTP server (`serve`)**: Bun/Alloy-style routing backed by Rocket.
- **Rocket Server Lifecycle (`Rocket`)**: Direct programmatic control over mounting routes, registering catchers, ignition, and graceful shutdown.
- **TypeScript Code Generation (`codegen`)**:
  - `parseRoute` / `parseRoutes`: Resolve route patterns into `RouteDef` structures with path/query parameters.
  - `toRocketPath`: Translate route patterns (`/users/:id/*`) into Rocket route syntax (`/users/<id>/<_..>`).
  - `generateTypes`: Generate TypeScript route maps, type aliases (`ParamsOf`, `QueryOf`, `BodyOf`, `ResponseOf`), and handler interfaces.
  - `generateClient`: Generate a strongly-typed `fetch` client SDK with Rocket's `uri!` runtime builder.
  - `generateModule`: Generate a complete TypeScript module combining route constants, types, `uri()` helper, and client SDK.

---

## Quick Start

### Basic Server (`serve`)

```ts
import { serve, response, responseJson, file } from "@alloy-ts/server";

const server = serve({
  routes: {
    "/api/status": response("OK"),
    "/users/:id": (req) => response(`Hello ${req.params?.id}`),
    "/api/posts": {
      GET: () => response("List posts"),
      POST: async () => responseJson({ created: true }),
    },
    "/favicon.ico": file("./favicon.ico"),
  },
  fetch() {
    return response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);
```

### Rocket Lifecycle (`Rocket`)

```ts
import { Rocket } from "@alloy-ts/server";

const rocket = new Rocket({ port: 8000 });

rocket.mount("/", "GET", "/hello", async (req) => {
  return { status: 200, body: Buffer.from("Hello from Rocket!") };
});

await rocket.launch();
```

---

## Code Generation (Codegen)

### Route Pattern Parsing & Translation

```ts
import { parseRoute, toRocketPath } from "@alloy-ts/server";

const route = parseRoute({
  method: "GET",
  path: "/users/:id/posts/*?<q>",
});

console.log(route.rocketPath); // "/users/<id>/posts/<_..>"
console.log(route.name); // "getUsersByIdPosts"

const rocketPattern = toRocketPath("/users/:id/*");
// "/users/<id>/<_..>"
```

### TypeScript Client SDK & Types Generation

```ts
import { parseRoutes, generateModule } from "@alloy-ts/server";

const routes = parseRoutes([
  {
    method: "GET",
    path: "/users/:id?<active>",
    response: "User",
  },
  {
    method: "POST",
    path: "/users",
    body: "CreateUser",
    response: "User",
  },
]);

const code = generateModule(routes, {
  clientName: "UserApiClient",
  baseUrl: "http://localhost:3000",
});

console.log(code);
```

---

## Development

- Configure local hooks:

```bash
npm run prepare
```

- Install dependencies:

```bash
npm install
```

- Run unit tests:

```bash
npm test
```

- Build library & native NAPI bindings:

```bash
npm run build
```

- Code formatting:

```bash
npm run fmt
```

- Linting:

```bash
npm run lint
```

- Code check:

```bash
npm run check
```

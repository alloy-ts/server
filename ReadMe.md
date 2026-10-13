# @alloy-ts/server

Node.js (TypeScript) NAPI bindings for the Rocket web framework and code generation utilities.

## Features

- **High-performance HTTP server (`serve`)**: Bun/Alloy-style routing backed by Rocket.
- **Rocket Server Lifecycle (`Rocket`)**: Direct programmatic control over creating Rocket instances, mounting routes, registering catchers, ignition, launching, and graceful shutdown.
- **TypeScript Code Generation (`codegen`)**:
  - `parseRoute` / `parseRoutes`: Resolve route strings (accepting both Bun `:id` and Rocket `<id>` syntax) into strongly-typed `RouteDef` structures with metadata, parameters, and handler names.
  - `toRocketPath`: Translate Bun/Alloy route syntax (`/users/:id/*`) into Rocket route syntax (`/users/<id>/<_..>`).
  - `httpMethods`: List HTTP methods supported by Rocket route attributes.
  - `generateTypes`: Emit `.d.ts` route maps (`RouteTypes`, `ParamsOf`, `QueryOf`, `BodyOf`, `ResponseOf`, `RouteContext`, `RouteHandler`).
  - `generateClient`: Emit a strongly-typed `fetch` client SDK alongside Rocket's `uri()` helper.
  - `generateModule`: Emit a full TypeScript module containing the `ROUTES` map, route type declarations, `uri()` helper, and client class.

---

## Quick Start

### Basic Server (`serve`)

```ts
import { serve, response, responseJson, file } from "@alloy-ts/server";

const server = serve(
  {
    "/api/status": response("OK"),
    "/users/:id": (req) => response(`Hello ${req.params.id}`),
    "/api/posts": {
      GET: () => response("List posts"),
      POST: async (req) => responseJson({ created: true }),
    },
    "/favicon.ico": file("./favicon.ico"),
  },
  (req) => response("Not Found", { status: 404 }),
  {
    port: 3000,
  },
);

console.log(`Server running at ${server.url}`);
```

### Programmatic Rocket Lifecycle (`Rocket`)

```ts
import { Rocket } from "@alloy-ts/server";

const rocket = new Rocket({
  port: 8000,
  logLevel: "normal",
});

rocket.mount("/", "GET", "/hello/<name>", async (req) => ({
  status: 200,
  headers: [],
  body: Buffer.from(`Hello, ${req.params?.name ?? "world"}`),
}));

await rocket.launch();
```

---

## Code Generation (Codegen)

### Route Pattern Parsing & Translation

```ts
import { parseRoute, parseRoutes, toRocketPath } from "@alloy-ts/server";

const route = parseRoute({
  method: "GET",
  path: "/users/:id/posts/*?<q>",
  response: "Post[]",
});

console.log(route.key); // "GET /users/:id/posts/*"
console.log(route.rocketPath); // "/users/<id>/posts/<_..>"
console.log(route.name); // "getUsersByIdPosts"

const rocketPath = toRocketPath("/users/:id/*");
// "/users/<id>/<_..>"
```

### TypeScript Client SDK & Type Generation

```ts
import { parseRoutes, generateModule } from "@alloy-ts/server";

const routes = parseRoutes([
  { method: "GET", path: "/users/:id", response: "User" },
  { method: "POST", path: "/users", body: "CreateUser", response: "User" },
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

- Linting & Type check:

```bash
npm run check
```

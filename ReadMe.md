# Server

Node.js (TypeScript) NAPI bindings for the Rocket web framework and code generation utilities.

## Features

- **High-performance HTTP server (`Alloy.serve` / `serve`)**: Bun/Alloy-style routing backed by Rocket.
- **Rocket Server Lifecycle (`Rocket`)**: Direct programmatic control over mounting routes, registering catchers, ignition, and graceful shutdown.
- **Rocket Configuration (`Config`)**: Full mirror of Rocket's server configuration (`Config.debugDefault()`, `Config.tryFrom()`, limits, shutdown signals, log levels).
- **TypeScript Code Generation (`codegen`)**:
  - `parseRocketRoutePattern`: Extract dynamic path/query parameters from Rocket route strings.
  - `translatePatternToAlloy`: Translate Rocket route syntax (`/users/<id>`) into Alloy/Bun route syntax (`/users/:id`).
  - `toRocketPath`: Translate Alloy/Bun route syntax (`/users/:id`) into Rocket route syntax (`/users/<id>`).
  - `generateTypescriptClient`: Generate a strongly-typed fetch client SDK for Rocket route specifications.
  - `generateRouteHandler`: Generate TypeScript route handler boilerplate.
  - `generateTypescriptInterface`: Convert struct/enum field definitions into TypeScript interfaces.
  - `generateModule` & `generateTypes`: Generate complete TypeScript route maps, handler types, and URL builders.

---

## Quick Start

### Basic Server (`serve`)

```ts
import { serve, response, responseJson, file } from "@alloy-ts/server";

const server = serve(
  {
    routes: {
      "/api/status": response("OK"),
      "/users/:id": (req) => response(`Hello ${req.params?.id}`),
      "/api/posts": {
        GET: () => response("List posts"),
        POST: async (req) => responseJson({ created: true }),
      },
      "/favicon.ico": file("./favicon.ico"),
    },
    fetch(req) {
      return response("Not Found", { status: 404 });
    },
  },
  undefined,
  { port: 3000 },
);

console.log(`Server running at ${server.url}`);
```

### Rocket Server Lifecycle (`Rocket`)

```ts
import { Rocket, Config } from "@alloy-ts/server";

const config = Config.debugDefault();
config.port = 8080;

const rocket = new Rocket({
  port: 8080,
  logLevel: "normal",
});

rocket.mount("/", "GET", "/hello/<name>", (req) => {
  return {
    status: 200,
    body: Buffer.from(`Hello, ${req.params?.name}!`),
  };
});

await rocket.launch();
```

---

## Code Generation (Codegen)

### Route Pattern Parsing & Translation

```ts
import { parseRocketRoutePattern, translatePatternToAlloy, toRocketPath } from "@alloy-ts/server";

const params = parseRocketRoutePattern("/users/<id>?<age>&<name..>");
// Extracted parameters:
// [
//   { name: "id", paramType: "string", isPath: true, isQuery: false, isOptional: false },
//   { name: "age", paramType: "string", isPath: false, isQuery: true, isOptional: false },
//   { name: "name", paramType: "string", isPath: false, isQuery: true, isOptional: true }
// ]

const alloyPattern = translatePatternToAlloy("/users/<id>/<rest..>");
// "/users/:id/*"

const rocketPattern = toRocketPath("/users/:id/*");
// "/users/<id>/<_..>"
```

### TypeScript Client SDK Generation

```ts
import { generateTypescriptClient } from "@alloy-ts/server";

const result = generateTypescriptClient(
  [
    {
      method: "GET",
      path: "/users/<id>",
      name: "getUser",
      params: [
        { name: "id", paramType: "string", isPath: true, isQuery: false, isOptional: false },
      ],
      returnType: "Promise<{ id: string; name: string }>",
    },
  ],
  { clientName: "UserApiClient" },
);

console.log(result.code);
```

### Route Handler & Interface Generation

```ts
import { generateRouteHandler, generateTypescriptInterface } from "@alloy-ts/server";

const handlerCode = generateRouteHandler({ method: "GET", path: "/users/:id" });
// Generates typed route handler boilerplate for GET /users/:id

const userInterface = generateTypescriptInterface("User", [
  { name: "id", tsType: "string", optional: false, description: "User ID" },
  { name: "name", tsType: "string", optional: false },
]);
// Generates:
// export interface User {
//   /** User ID */
//   id: string;
//   name: string;
// }
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

# Rocket Web Framework for TypeScript (Node.js) - User Manual

`@alloy-ts/server` provides high-performance Node.js NAPI bindings for the Rust [Rocket](https://rocket.rs) web framework v0.5. It brings Rocket's type-safe routing, request handling, configuration management, and code generation utilities directly to TypeScript and JavaScript runtime environments.

---

## Table of Contents

1. [Installation](#installation)
2. [Quick Start](#quick-start)
3. [Server & Web Framework](#server--web-framework)
   - [Single-Function Server (`Alloy.serve`)](#single-function-server-alloyserve)
   - [Builder API (`Alloy.app`)](#builder-api-alloyapp)
   - [Domain / Socket Support (Unix Sockets & Port 0)](#domain--socket-support-unix-sockets--port-0)
4. [Routing & URIs](#routing--uris)
   - [Route (`Alloy.Route`)](#route-alloyroute)
   - [RouteUri (`Alloy.RouteUri`)](#routeuri-alloyrouteuri)
5. [Handlers & Catchers](#handlers--catchers)
   - [Handler (`Alloy.Handler`)](#handler-alloyhandler)
   - [Catcher (`Alloy.Catcher`)](#catcher-alloycatcher)
6. [Configuration (`Alloy.Config`)](#configuration-alloyconfig)
7. [Code Generation (`Alloy.codegen`)](#code-generation-alloycodegen)

---

## Installation

Ensure you have `@alloy-ts/server` installed in your project:

```bash
npm install @alloy-ts/server
```

---

## Quick Start

Create an HTTP server with custom routes and response handling using `Alloy.serve`:

```typescript
import { Alloy } from "@alloy-ts/server";

const server = Alloy.serve({
  port: 8080,
  routes: [
    {
      method: "GET",
      path: "/",
      handler: (req) => "Hello from Rocket TypeScript!",
    },
    {
      method: "POST",
      path: "/api/echo",
      handler: (req) => ({
        status: 200,
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ echo: req.body }),
      }),
    },
  ],
});

console.log(`Server listening at http://${server.hostname}:${server.port}`);
```

---

## Server & Web Framework

### Single-Function Server (`Alloy.serve`)

`Alloy.serve` launches a Rocket web server instantly. It supports both positional arguments and a single options object overload.

```typescript
// Single Options Object
const server = Alloy.serve({
  hostname: "127.0.0.1",
  port: 3000,
  routes: [{ method: "GET", path: "/health", handler: () => "OK" }],
  fetch: (req) => "Fallback Handler",
});

// Stop the server when needed
server.stop();
```

### Builder API (`Alloy.app`)

For a fluent API similar to Express or Rocket's builder pattern:

```typescript
const app = Alloy.app();

app
  .get("/", (req) => "Welcome Home")
  .post("/submit", (req) => "Submitted")
  .catch(404, (req) => "Custom 404 Page");

const server = app.launch({ port: 8080 });
```

### Domain / Socket Support (Unix Sockets & Port 0)

- **Random Port (`port: 0`):** Rocket dynamically binds to an available OS port. The returned server instance exposes the bound port immediately via `server.port`.
- **Unix Domain Sockets (`unix`):** Bind Rocket directly to a Unix domain socket path:
  ```typescript
  const server = Alloy.serve({
    unix: "/tmp/rocket.sock",
    routes: [{ method: "GET", path: "/", handler: () => "Socket connected" }],
  });
  ```

---

## Routing & URIs

### Route (`Alloy.Route`)

Wrap low-level Rocket route definitions with ranking calculation and base path re-mapping:

```typescript
import { Route, Method } from "@alloy-ts/server";

const route = new Route(Method.Get, "/users/<id>", (req) => "User details");
console.log(route.rank); // Calculates Rocket rank automatically
route.mapBase("/api/v1");
console.log(route.uri); // "/api/v1/users/<id>"
```

### RouteUri (`Alloy.RouteUri`)

Parse and inspect Rocket route URIs with base and dynamic parameter validation:

```typescript
import { RouteUri } from "@alloy-ts/server";

const uri = new RouteUri("/api/v1", "/users/<id>?<details..>");
console.log(uri.base); // "/api/v1"
console.log(uri.path); // "/users/<id>"
console.log(uri.query); // "<details..>"
```

---

## Handlers & Catchers

### Handler (`Alloy.Handler`)

Rocket handlers process incoming requests and return string, buffer, or response object structures.

```typescript
import { Handler } from "@alloy-ts/server";

const handler = new Handler((req) => {
  return {
    status: 201,
    headers: { "x-custom-header": "alloy" },
    body: "Created",
  };
});

const route = handler.toRoute("POST", "/create");
```

### Catcher (`Alloy.Catcher`)

Catch HTTP error status codes (e.g., 400–599) or set a default error catcher:

```typescript
import { Catcher } from "@alloy-ts/server";

const notFoundCatcher = new Catcher(404, (req) => "404 Not Found");
const defaultCatcher = Catcher.defaultCatcher((req) => "An error occurred");
```

---

## Configuration (`Alloy.Config`)

Configure Rocket web server parameters (IP, port, workers, limits, profile, keep-alive):

```typescript
import { Config, Profile } from "@alloy-ts/server";

const config = new Config(Profile.Debug);
config.port = 8080;
config.address = "0.0.0.0";
config.workers = 4;
config.keepAlive = 5;

const validated = config.extract(); // Validates parameters
```

---

## Code Generation (`Alloy.codegen`)

`Alloy.codegen` converts route specifications into type-safe TypeScript interfaces, client request callers, and server module definitions:

```typescript
import { parseRoutes, generateTypes, generateClient, generateModule } from "@alloy-ts/server";

const routes = parseRoutes(["GET /users/<id>?<details..>", "POST /users"]);

const types = generateTypes(routes);
const client = generateClient(routes);
const moduleCode = generateModule(routes);
```

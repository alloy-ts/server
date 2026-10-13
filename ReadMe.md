# @alloy-ts/server

TypeScript and Node.js NAPI bindings for the [Rocket](https://rocket.rs/) web framework.

## Overview

`@alloy-ts/server` provides high-performance Node.js bindings for Rocket, bringing Rocket's multi-threaded Tokio runtime, robust server configuration, response helpers, and URI code-generation utilities to TypeScript and Node.js.

## Features

- **High-Performance HTTP Server (`Alloy.serve` / `serve`)**: Bridge JavaScript route handlers directly into Rocket handlers executed on multi-threaded Tokio worker threads.
- **Rocket Application Lifecycle (`Rocket`)**: Configure, mount routes and catchers, ignite, launch, and gracefully shut down Rocket servers programmatically from TypeScript.
- **Server Configuration (`Config`)**: Full mirror of Rocket's configuration options (port, address, workers, limits, TLS, shutdown, log level, figment provider).
- **Code Generation & URI Helpers (`codegen` / `uri`)**: Type-safe and URL-safe URI generation for dynamic route patterns.
- **Rich Response Helpers**: Bun-style `response`, `responseJson`, `responseRedirect`, and `file` helpers.

## Installation & Usage

```ts
import { serve, response, responseJson, uri, Config, Rocket } from "@alloy-ts/server";

// High-performance server definition
const server = serve(
  {
    "/": () => response("Hello from Rocket!"),
    "/users/:id": (req) => responseJson({ userId: req.params.id }),
  },
  undefined,
  { port: 8080 },
);

console.log(`Server running at ${server.url}`);

// Codegen helper for route URIs
const userUri = uri("/users/:id", { id: "42" }, { ref: "nav" });
console.log(userUri); // "/users/42?ref=nav"
```

## Development

- Install dependencies:

```bash
npm install
```

- Build native NAPI bindings:

```bash
npm run build:napi
```

- Run unit tests:

```bash
npm run test
```

- Code formatting & linting:

```bash
npm run fmt
npm run lint
npm run check
```

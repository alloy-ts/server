# @alloy-ts/server

Node.js / TypeScript NAPI bindings for the [Rocket web framework](https://rocket.rs/).

High-performance HTTP server backed by Rust's Rocket web framework with NAPI bindings and code generation support for Node.js / TypeScript.

## Features

- **Rocket Web Framework Native Bindings**: Driven by Rust's Rocket web framework on a multi-threaded Tokio runtime.
- **`Alloy.serve` API**: Support for route tables, dynamic parameters (`:id`, `<param>`), wildcards, static file serving, and custom fetch handlers.
- **Code Generation (`Codegen`)**: NAPI utilities for route metadata extraction, route pattern parsing (`parseRoutePattern`), and TypeScript route code generation.

## Usage

```ts
import { Alloy, Codegen, parseRoutePattern } from "@alloy-ts/server";

// Start a Rocket-backed server
const server = Alloy.serve({
  port: 3000,
  routes: {
    "/api/status": new Response("OK"),
    "/users/:id": (req) => new Response(`User ${req.params.id}`),
    "/favicon.ico": Alloy.file("./favicon.ico"),
  },
  fetch(req) {
    return new Response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);

// Use Codegen utilities
const codegen = new Codegen();
codegen.addRoute({
  method: "GET",
  path: "/api/hello/<name>",
  name: "hello",
  queryParams: [],
  pathParams: ["name"],
});

const output = codegen.generateTs();
console.log(output.tsCode);
```

## Development

- Install dependencies:

```bash
npm install
```

- Build NAPI native binary:

```bash
npm run build:napi
```

- Run unit tests:

```bash
npm test
```

- Code formatting & linting:

```bash
npm run check
```

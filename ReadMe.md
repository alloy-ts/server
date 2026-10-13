# Rocket NAPI Bindings & TypeScript Codegen

NAPI bindings bridging Rust's [Rocket](https://rocket.rs/) web framework to Node.js and TypeScript.

## Features

- **High-Performance Server (`Alloy.serve`)**: Host Bun/Web-style route tables driven by Rocket's multi-threaded runtime.
- **TypeScript Codegen & URI Generation (`Alloy.uri`)**: Type-safe URI path generation with URL-safe dynamic parameter encoding.
- **Response Builders (`Alloy.Response`)**: Construct text, JSON (`Response.json`), and redirect (`Response.redirect`) responses.
- **Static File Handling (`Alloy.file`)**: Lazily serve static files directly from disk.
- **Rocket Server Lifecycle (`Alloy.Rocket` / `Alloy.Config`)**: Programmatically build, configure, mount, register catchers, and launch Rocket servers.

## Quick Start

```ts
import { Alloy } from "@alloy-ts/server";

const server = Alloy.serve({
  port: 3000,
  routes: {
    "/api/status": Alloy.Response.json({ status: "ok" }),
    "/users/:id": (req) => {
      return new Alloy.Response(`Hello User ${req.params.id}!`);
    },
    "/favicon.ico": Alloy.file("./favicon.ico"),
  },
  fetch(req) {
    return new Alloy.Response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);
```

## Development

- Install dependencies:

```bash
npm install
```

- Build NAPI native bindings:

```bash
npm run build:napi
```

- Run unit tests:

```bash
npx vp test
```

- Code formatting:

```bash
npm run fmt
```

- Linting:

```bash
npm run lint
```

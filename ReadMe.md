# Alloy Server Native

High-performance NAPI bindings for Rocket web framework in TypeScript (Node.js).

## Usage

```ts
import { serve, Response, file } from "@alloy-ts/server";

const server = serve({
  port: 3000,
  routes: {
    "/api/status": new Response("OK"),
    "/users/:id": (req) => {
      return new Response(`Hello User ${req.params.id}!`);
    },
    "/api/posts": {
      GET: () => new Response("List posts"),
      POST: async (req) => {
        const data = JSON.parse(req.body?.toString("utf-8") ?? "{}");
        return Response.json({ created: true, ...data });
      },
    },
    "/favicon.ico": file("./favicon.ico"),
  },
  fetch(req) {
    return new Response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);
```

## Development

- Configure local hooks:

```bash
npm run prepare
```

- Install dependencies:

```bash
vp install
```

- Run the unit tests:

```bash
vp test
```

- Build the library:

```bash
npm run build
```

- Code check:

```bash
npm run check
```

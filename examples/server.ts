import { Alloy, Response } from "../dist/index.js";

// Start a high-performance HTTP server backed by Rocket web framework NAPI bindings
const server = Alloy.serve({
  port: 3000,
  hostname: "localhost",
  routes: {
    // Static routes
    "/api/status": new Response("OK"),

    // Dynamic routes
    "/users/:id": (req: any) => {
      return new Response(`Hello User ${req.params.id}!`);
    },

    // Per-HTTP method handlers
    "/api/posts": {
      GET: () => new Response("List posts"),
      POST: async (req: any) => {
        const bodyStr = req.body ? new TextDecoder().decode(new Uint8Array(req.body)) : "{}";
        const body = JSON.parse(bodyStr);
        return Response.json({ created: true, ...body });
      },
    },

    // Wildcard route for all routes that start with "/api/" and aren't otherwise matched
    "/api/*": Response.json({ message: "Not found" }, { status: 404 }),

    // Redirect from /blog/hello to /blog/hello/world
    "/blog/hello": Response.redirect("/blog/hello/world"),

    // Serve a file by lazily loading it into memory
    "/favicon.ico": Alloy.file("./favicon.ico"),
  },

  // Optional fallback for unmatched routes
  fetch(_req: any) {
    return new Response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);

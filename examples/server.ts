import { Alloy, Response, file, serve } from "../src/main.ts";

// Start a high-performance Rocket-backed server with Bun/Alloy-compatible API
const server = Alloy.serve({
  port: 3000,
  hostname: "0.0.0.0",

  routes: {
    // Static response routes
    "/api/status": new Response("OK"),

    // Dynamic routes with dynamic path parameters
    "/users/:id": (req: any) => {
      return new Response(`Hello User ${req.params?.id || "Unknown"}!`);
    },

    // Per-HTTP method handlers
    "/api/posts": {
      GET: () => new Response("List posts"),
      POST: async (req: any) => {
        const body = req.body ? JSON.parse(Buffer.from(req.body).toString("utf-8")) : {};
        return Response.json({ created: true, ...body });
      },
    },

    // Wildcard route for unmatched /api/* endpoints
    "/api/*": Response.json({ message: "Not found" }, { status: 404 }),

    // Redirect route
    "/blog/hello": Response.redirect("/blog/hello/world"),

    // Serve a static file lazily
    "/favicon.ico": file("./favicon.ico"),
  },

  // Fallback handler for unmatched routes
  fetch(_req: any) {
    return new Response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url} (port ${server.port})`);

// Graceful shutdown after demo run
setTimeout(() => {
  console.log("Shutting down server...");
  server.stop();
}, 1000);

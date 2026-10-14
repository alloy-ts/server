import { serve, file, Response } from "../dist/index.js";

// Simulated HTML import asset manifest (e.g., imported index.html)
const myReactSinglePageApp = {
  html: "<!DOCTYPE html><html><body><h1>React SPA</h1></body></html>",
};

const server = serve({
  port: Number(process.env.PORT || 3000),
  hostname: "127.0.0.1",

  routes: {
    // 1. Static routes
    "/api/status": new Response("OK"),

    // 2. Dynamic routes
    "/users/:id": (req: any) => {
      return new Response(`Hello User ${req.params.id}!`);
    },

    // 3. Per-HTTP method handlers
    "/api/posts": {
      GET: () => new Response("List posts"),
      POST: async (req: any) => {
        const body = req.body ? JSON.parse(req.body.toString()) : {};
        return Response.json({ created: true, ...body });
      },
    },

    // 4. Wildcard matching for unmatched /api/* requests
    "/api/*": Response.json({ message: "Not found" }, { status: 404 }),

    // 5. Redirects
    "/blog/hello": Response.redirect("/blog/hello/world"),

    // 6. Serve a file by lazily loading it from disk
    "/favicon.ico": file("./favicon.ico"),

    // 7. HTML imports / single page app entrypoint
    "/": myReactSinglePageApp,
  },

  // Fallback for unmatched routes
  fetch(_req: any) {
    return new Response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);

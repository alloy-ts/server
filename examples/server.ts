import { Alloy, Response } from "../src/main.ts";

const server = Alloy.serve({
  port: 3000,
  routes: {
    // Static routes
    "/api/status": Alloy.Response("OK"),

    // Dynamic routes
    "/users/:id": (req) => {
      return Alloy.Response(`Hello User ${req.params.id}!`);
    },

    // Per-HTTP method handlers
    "/api/posts": {
      GET: () => Alloy.Response("List posts"),
      POST: async (req) => {
        const body = req.body ? JSON.parse(Buffer.from(req.body).toString()) : {};
        return Alloy.Response.json({ created: true, ...body });
      },
    },

    // Wildcard route for all routes that start with "/api/" and aren't otherwise matched
    "/api/*": Alloy.Response.json({ message: "Not found" }, { status: 404 }),

    // Redirect from /blog/hello to /blog/hello/world
    "/blog/hello": Alloy.Response.redirect("/blog/hello/world"),

    // Serve a file by lazily loading it into memory
    "/favicon.ico": Alloy.file("./favicon.ico"),
  },

  // Fallback for unmatched routes
  fetch(req) {
    return Alloy.Response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);

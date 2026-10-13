import { Alloy, response, responseJson, responseRedirect } from "../src/main.ts";

// Start a high-performance HTTP server backed by Rocket NAPI bindings
const server = Alloy.serve({
  port: 8080,
  hostname: "0.0.0.0",

  routes: {
    // Static routes
    "/api/status": response("OK"),

    // Dynamic routes with path parameters
    "/users/:id": (req: any) => {
      return response(`Hello User ${req.params?.id || "unknown"}!`);
    },

    // Per-HTTP method handlers
    "/api/posts": {
      GET: () => responseJson([{ id: 1, title: "First Post" }]),
      POST: async (req: any) => {
        return responseJson({ created: true, body: req.body }, { status: 201 });
      },
    },

    // Wildcard route for all routes that start with "/api/" and aren't otherwise matched
    "/api/*": responseJson({ message: "Not found" }, { status: 404 }),

    // Redirect from /blog/hello to /blog/hello/world
    "/blog/hello": responseRedirect("/blog/hello/world"),

    // Serve a file lazily
    "/favicon.ico": Alloy.file("./favicon.ico"),
  },

  // Fallback for unmatched routes
  fetch(_req: any) {
    return response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);

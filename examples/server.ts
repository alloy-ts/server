import { file, response, responseJson, responseRedirect, serve } from "../dist/index.js";

const server = serve({
  port: 3000,
  routes: {
    // Static routes
    "/api/status": response("OK"),

    // Dynamic routes
    "/users/:id": (req) => {
      return response(`Hello User ${req.params.id}!`);
    },

    // Per-HTTP method handlers
    "/api/posts": {
      GET: () => response("List posts"),
      POST: async () => {
        return responseJson({ created: true });
      },
    },

    // Wildcard route for all routes that start with "/api/" and aren't otherwise matched
    "/api/*": responseJson({ message: "Not found" }, { status: 404 }),

    // Redirect from /blog/hello to /blog/hello/world
    "/blog/hello": responseRedirect("/blog/hello/world"),

    // Serve a file by lazily loading it into memory
    "/favicon.ico": file("./favicon.ico"),
  },

  // (optional) fallback for unmatched routes
  fetch() {
    return response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);

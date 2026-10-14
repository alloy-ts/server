import { serve, response, responseJson, responseRedirect, file } from "../dist/index.js";

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
      POST: async (req) => {
        const body = req.body ? JSON.parse(Buffer.from(req.body).toString()) : {};
        return responseJson({ created: true, ...body });
      },
    },

    // Wildcard route for all routes that start with "/api/" and aren't otherwise matched
    "/api/*": responseJson({ message: "Not found" }, { status: 404 }),

    // Redirect from /blog/hello to /blog/hello/world
    "/blog/hello": responseRedirect("/blog/hello/world"),

    // Serve a file by lazily loading it into memory
    "/favicon.ico": file("./favicon.ico"),
  },

  // Fallback for unmatched routes
  fetch(req) {
    return response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);

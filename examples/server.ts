import { serve, response, responseJson, responseRedirect, file } from "@alloy-ts/server";

const server = serve({
  routes: {
    // Static response
    "/api/status": response("OK"),

    // Dynamic parameter route
    "/users/:id": async (req) => {
      return response(`Hello User ${req.params["id"]}!`);
    },

    // Per-HTTP method handlers
    "/api/posts": {
      GET: async () => response("List posts"),
      POST: async (req) => {
        return responseJson({ created: true, bodyLength: req.body?.length ?? 0 });
      },
    },

    // Redirect
    "/blog/hello": responseRedirect("/blog/hello/world"),

    // File serving
    "/favicon.ico": file("./favicon.ico"),
  },

  // Fallback for unmatched routes
  fetch(req) {
    return response(`Not Found: ${req.path}`, { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);

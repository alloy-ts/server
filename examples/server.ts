import { file, response, responseJson, responseRedirect, serve } from "../dist/index.js";

// Alloy.serve compatible API powered by Rocket web framework NAPI bindings
const server = serve(
  {
    // Static routes
    "/api/status": response("OK"),

    // Dynamic routes with path parameters
    "/users/:id": (req) => {
      return response(`Hello User ${req.params.id}!`);
    },

    // Per-HTTP method handlers
    "/api/posts": {
      GET: () => response("List posts"),
      POST: (req) => {
        let body = {};
        if (req.body) {
          try {
            body = JSON.parse(Buffer.from(req.body).toString("utf-8"));
          } catch {
            // fallback
          }
        }
        return responseJson({ created: true, ...body });
      },
    },

    // Wildcard route for all routes starting with /api/
    "/api/*": responseJson({ message: "Not found" }, { status: 404 }),

    // Redirect route
    "/blog/hello": responseRedirect("/blog/hello/world"),

    // Serve a file lazily
    "/favicon.ico": file("./favicon.ico"),
  },

  // Fallback handler for unmatched routes
  (_req) => {
    return response("Not Found", { status: 404 });
  },

  // Server configuration options
  {
    port: 3000,
    hostname: "0.0.0.0",
  },
);

console.log(`Rocket HTTP Server running at ${server.url}`);

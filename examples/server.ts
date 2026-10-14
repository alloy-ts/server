import { file, response, responseJson, responseRedirect, serve } from "../dist/index.js";

const server = serve({
  port: Number(process.env["PORT"] ?? 3000),
  hostname: "127.0.0.1",
  routes: {
    // Static routes
    "/api/status": response("OK"),

    // Dynamic routes
    "/users/:id": (req) => {
      return response(`Hello User ${req.params["id"] ?? "unknown"}!`);
    },

    // Per-HTTP method handlers
    "/api/posts": {
      GET: () => response("List posts"),
      POST: async (req) => {
        const bodyText = req.body ? new TextDecoder().decode(Uint8Array.from(req.body)) : "{}";
        const body = JSON.parse(bodyText) as Record<string, unknown>;
        return responseJson({ created: true, ...body });
      },
    },

    // Wildcard route
    "/api/*": responseJson({ message: "Not found" }, { status: 404 }),

    // Redirect
    "/blog/hello": responseRedirect("/blog/hello/world"),

    // Serve static file
    "/favicon.ico": file("./favicon.ico"),
  },

  // Fallback for unmatched routes
  fetch(req) {
    return response(`Not Found: ${req.path}`, { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);

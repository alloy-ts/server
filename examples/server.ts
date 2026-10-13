import { Alloy, Bun, response, responseJson, responseRedirect } from "../src/index.ts";

const server = Alloy.serve({
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
        let body: Record<string, unknown> = {};
        if (req.body) {
          try {
            body = JSON.parse(Buffer.from(req.body).toString("utf-8"));
          } catch {
            // ignore JSON parse error
          }
        }
        return responseJson({ created: true, ...body });
      },
    },

    // Wildcard route for all routes that start with "/api/" and aren't otherwise matched
    "/api/*": responseJson({ message: "Not found" }, { status: 404 }),

    // Redirect from /blog/hello to /blog/hello/world
    "/blog/hello": responseRedirect("/blog/hello/world"),

    // Serve a file by lazily loading it into memory
    "/favicon.ico": Bun.file("./favicon.ico"),
  },

  // (optional) fallback for unmatched routes:
  fetch(_req) {
    return response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);

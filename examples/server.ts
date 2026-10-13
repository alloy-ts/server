import { Alloy, Bun, response, responseJson, responseRedirect } from "../dist/index.js";

// Use Alloy.serve to start a high-performance HTTP server powered by Rocket NAPI bindings
const server = Alloy.serve({
  port: Number(process.env.PORT) || 3000,
  hostname: "0.0.0.0",

  routes: {
    // Static routes
    "/api/status": response("OK"),

    // Dynamic routes
    "/users/:id": (req: any) => {
      return response(`Hello User ${req.params.id}!`);
    },

    // Per-HTTP method handlers
    "/api/posts": {
      GET: () => response("List posts"),
      POST: async (req: any) => {
        const body = req.json();
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

  // Fallback handler for unmatched routes
  fetch(_req: any) {
    return response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);

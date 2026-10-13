import { serve, response, responseJson, responseRedirect, file } from "../dist/index.js";

// Demonstrate Bun.serve / Alloy.serve compatible API backed by Rocket web framework
const server = serve(
  {
    // Static routes
    "/api/status": response("OK"),

    // Dynamic routes
    "/users/:id": (req: { params?: { id?: string } }) => {
      return response(`Hello User ${req.params?.id}!`);
    },

    // Per-HTTP method handlers
    "/api/posts": {
      GET: () => response("List posts"),
      POST: async (req: { body?: Uint8Array }) => {
        const text = req.body ? new TextDecoder().decode(new Uint8Array(req.body)) : "{}";
        let parsed = {};
        try {
          parsed = JSON.parse(text);
        } catch {
          // fallback
        }
        return responseJson({ created: true, ...parsed });
      },
    },

    // Wildcard route for all routes that start with "/api/" and aren't otherwise matched
    "/api/*": responseJson({ message: "Not found" }, { status: 404 }),

    // Redirect from /blog/hello to /blog/hello/world
    "/blog/hello": responseRedirect("/blog/hello/world"),

    // Serve a file lazily
    "/favicon.ico": file("./favicon.ico"),
  },

  // Fallback for unmatched routes
  (_req: unknown) => {
    return response("Not Found", { status: 404 });
  },

  // Options
  {
    port: 3000,
    hostname: "0.0.0.0",
  },
);

console.log(`Rocket server running at ${server.url}`);

if (process.env.NODE_ENV === "test") {
  server.stop();
}

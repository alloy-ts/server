import {
  Alloy,
  response,
  responseJson,
  responseRedirect,
  file,
  RequestInput,
} from "../dist/index.js";

// Alloy.serve starts a high-performance HTTP server backed by Rocket (NAPI)
const server = Alloy.serve({
  // Port defaults to $BUN_PORT, $PORT, $NODE_PORT or 3000
  port: Number(process.env.PORT) || 3000,
  hostname: "127.0.0.1",

  routes: {
    // Static response routes
    "/api/status": response("OK"),

    // Dynamic path segment routes
    "/users/:id": (req: RequestInput) => {
      return response(`Hello User ${req.params.id}!`);
    },

    // Per-HTTP method handlers
    "/api/posts": {
      GET: () => response("List posts"),
      POST: async (req: RequestInput) => {
        const bodyStr = req.body ? Buffer.from(req.body).toString("utf-8") : "";
        const body = bodyStr ? JSON.parse(bodyStr) : {};
        return responseJson({ created: true, ...body });
      },
    },

    // Wildcard route for unmatched /api/* paths
    "/api/*": responseJson({ message: "Not found" }, { status: 404 }),

    // Redirect route
    "/blog/hello": responseRedirect("/blog/hello/world"),

    // Lazy file route
    "/favicon.ico": file("./favicon.ico"),

    // HTML response
    "/": "<h1>Welcome to Alloy Web Server!</h1>",
  },

  // Fallback for unmatched routes
  fetch(req: RequestInput) {
    return responseJson({ error: "Not Found", path: req.path }, { status: 404 });
  },
});

console.log(`Server running at ${server.url} (port: ${server.port})`);

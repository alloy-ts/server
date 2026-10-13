import { file, response, responseJson, responseRedirect, serve } from "@alloy-ts/server";

// Express / Bun compatible HTTP server backed by Rocket
const server = serve(
  {
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
        const bodyText = req.body ? new TextDecoder().decode(new Uint8Array(req.body)) : "{}";
        const body = JSON.parse(bodyText);
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

  // Fallback handler for unmatched routes
  (_req) => {
    return response("Not Found", { status: 404 });
  },

  // Server configuration options
  {
    port: 3000,
    hostname: "127.0.0.1",
  },
);

console.log(`Rocket server running at ${server.url}`);

// Stop server gracefully when done
setTimeout(() => {
  server.stop();
  console.log("Server stopped gracefully");
}, 1000);

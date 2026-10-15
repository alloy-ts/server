import { serve, response, responseJson, responseRedirect } from "../dist/index.js";

/**
 * Demonstrates the Bun / Alloy compatible HTTP server API using Rocket.
 */
const server = serve({
  routes: {
    "/": response("Hello from Rocket Web Framework!"),

    "/api/status": responseJson({ ok: true, framework: "Rocket" }),

    "/users/:id": (req) => {
      return responseJson({ userId: req.params?.id });
    },

    "/redirect": responseRedirect("/api/status"),
  },

  fetch() {
    return response("Not Found", { status: 404 });
  },

  port: 8080,
  hostname: "127.0.0.1",
});

console.log(`Rocket server running at ${server.url}`);
server.stop();

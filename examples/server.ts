/**
 * Bun.serve / Alloy.serve compatible HTTP server backed by Rocket NAPI bindings.
 */

import { file, response, serve } from "@alloy-ts/server";

const server = serve(
  {
    // Static routes
    "/api/status": response("OK"),

    // Dynamic routes
    "/users/:id": (req) => {
      return response(`Hello User ${req.params?.id}!`);
    },

    // Per-HTTP method handlers
    "/api/posts": {
      GET: () => response("List posts"),
      POST: async (req) => {
        const bodyText = req.body ? new TextDecoder().decode(new Uint8Array(req.body)) : "{}";
        const body = JSON.parse(bodyText);
        return response(JSON.stringify({ created: true, ...body }), {
          headers: { "Content-Type": "application/json" },
        });
      },
    },

    // Redirect route
    "/blog/hello": response(null, {
      status: 302,
      headers: { Location: "/blog/hello/world" },
    }),

    // Lazy static file route
    "/favicon.ico": file("./favicon.ico"),
  },

  // Fallback for unmatched routes
  (_req) => {
    return response("Not Found", { status: 404 });
  },

  // Server options
  {
    port: 3000,
    hostname: "0.0.0.0",
  },
);

console.log(`Server running at ${server.url}`);

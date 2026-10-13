/**
 * Example server using Rocket web framework NAPI bindings and Bun/Alloy-style API.
 *
 * Demonstrates:
 * - Static & dynamic routes (`/api/status`, `/users/:id`)
 * - Per-HTTP method handlers (`GET`, `POST`)
 * - Wildcard routes (`/api/*`)
 * - Redirect responses
 * - File responses
 * - Options (port, hostname)
 */

import { file, response, responseJson, responseRedirect, serve } from "../dist/index.js";

const server = serve(
  {
    // Static route
    "/api/status": response("OK"),

    // Dynamic route
    "/users/:id": (req: any) => {
      return response(`Hello User ${req.params?.id}!`);
    },

    // Per-HTTP method handlers
    "/api/posts": {
      GET: () => response("List posts"),
      POST: async () => {
        return responseJson({ created: true });
      },
    },

    // Wildcard route for unmatched /api/* endpoints
    "/api/*": responseJson({ message: "Not found" }, { status: 404 }),

    // Redirect
    "/blog/hello": responseRedirect("/blog/hello/world"),

    // File
    "/favicon.ico": file("./favicon.ico"),
  },

  // Fallback for unmatched routes
  () => response("Not Found", { status: 404 }),

  {
    port: 3000,
    hostname: "0.0.0.0",
  },
);

console.log(`Server running at ${server.url}`);

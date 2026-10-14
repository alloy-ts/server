/**
 * Example: Wildcards, Dynamic Routes, and Per-Method Handlers
 *
 * Demonstrates:
 * 1. Dynamic route parameters (`/users/:id`)
 * 2. Per-HTTP method route handlers (`GET`, `POST`, `PUT`, `DELETE`)
 * 3. Wildcard prefix matching (`/api/*`)
 * 4. Redirect responses
 */

import { Alloy, Response } from "../dist/main.mjs";

const server = Alloy.serve({
  routes: {
    // 1. Dynamic path parameter
    "/users/:id": (req: any) => {
      return new Response(`User ID: ${req.params.id}`);
    },

    // 2. Per-HTTP method map
    "/api/posts": {
      GET: () =>
        Response.json([
          { id: 1, title: "First Post" },
          { id: 2, title: "Second Post" },
        ]),
      POST: async (req: any) => {
        return Response.json({ created: true, body: req.body }, { status: 201 });
      },
    },

    // 3. Wildcard fallback for all unmatched /api/ routes
    "/api/*": Response.json({ error: "API Route Not Found" }, { status: 404 }),

    // 4. Redirect
    "/old-link": Response.redirect("/api/status", 301),
  },

  // Fallback for all other routes
  fetch() {
    return new Response("Fallback 404 Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);

setTimeout(() => {
  server.stop();
}, 2000);

import { Alloy, Response } from "../dist/index.js";

/**
 * Example demonstrating authorization, request header inspection,
 * and returning custom HTTP statuses.
 */

// Simulated token validator
function isValidToken(authHeader?: string): boolean {
  if (!authHeader) return false;
  return authHeader.startsWith("Bearer secret-token-123");
}

const server = Alloy.serve({
  port: 3005,
  routes: {
    // Public endpoint
    "/api/public": () => {
      return Response.json({
        message: "This is a public endpoint accessible by anyone.",
      });
    },

    // Protected user profile endpoint
    "/api/profile": (req) => {
      const authHeader = req.headers["authorization"];

      if (!authHeader) {
        return Response.json(
          { error: "Missing Authorization header" },
          { status: 401, headers: { "WWW-Authenticate": "Bearer" } },
        );
      }

      if (!isValidToken(authHeader)) {
        return Response.json({ error: "Forbidden: Invalid or expired token" }, { status: 403 });
      }

      return Response.json({
        id: "usr_9876",
        username: "alice",
        role: "admin",
      });
    },

    // Role-restricted admin endpoint
    "/api/admin/dashboard": (req) => {
      const authHeader = req.headers["authorization"];
      if (!isValidToken(authHeader)) {
        return Response.json({ error: "Unauthorized" }, { status: 401 });
      }

      return Response.json({
        metrics: {
          activeUsers: 1420,
          systemStatus: "Healthy",
          uptimeSeconds: 86400,
        },
      });
    },
  },

  fetch(req) {
    return Response.json({ error: "Not Found", path: req.url }, { status: 404 });
  },
});

console.log(`Auth & Guards Example Server running at ${server.url}`);

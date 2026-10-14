/**
 * Routes and Error Catchers Example
 *
 * Demonstrates:
 * - Creating a Rocket server instance
 * - Using `Route` and `RouteUri` to inspect default ranks and bases
 * - Mounting handlers with explicit routes
 * - Registering custom status code error catchers (e.g. 404, 500)
 */

import {
  handlerOutcomeError,
  handlerOutcomeSuccess,
  Rocket,
  Route,
  RouteUri,
} from "../dist/index.js";

// Inspect route ranks and URIs
const indexRoute = new Route("GET", "/api/users/:id");
console.log("=== Route Metadata ===");
console.log("Method:", indexRoute.method);
console.log("URI:   ", indexRoute.uri);
console.log("Rank:  ", indexRoute.rank); // Default rank computed by Rocket

const mappedRoute = indexRoute.mapBase("/v1");
console.log("Mapped Base URI:", mappedRoute.uri);

const routeUri = new RouteUri("/v1", "/users/<id>?active=true");
console.log("\n=== RouteUri Details ===");
console.log("Base:   ", routeUri.base);
console.log("Path:   ", routeUri.path());
console.log("Query:  ", routeUri.query());
console.log("Full:   ", routeUri.asStr());

// Initialize Rocket server instance
const rocket = new Rocket({
  port: 8080,
  address: "127.0.0.1",
  logLevel: "normal",
});

// Mount custom handlers
rocket.mount("/", "GET", "/api/hello/<name>", async (req: any) => {
  const name = req.params?.name ?? "world";
  return {
    status: 200,
    headers: [{ name: "Content-Type", value: "text/plain" }],
    body: Buffer.from(`Hello, ${name}!`),
  };
});

// Register a custom 404 Catcher
rocket.register("/", 404, async (req: any) => {
  return {
    status: 404,
    headers: [{ name: "Content-Type", value: "application/json" }],
    body: Buffer.from(
      JSON.stringify({
        error: "Not Found",
        path: req.path,
      }),
    ),
  };
});

console.log("\nRocket server configured with custom routes and catchers.");

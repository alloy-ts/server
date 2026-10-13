/**
 * Example: Using `@alloy-ts/server` Codegen Utilities & Type Generation
 *
 * Demonstrates:
 * 1. Type-safe URI construction (`uri`)
 * 2. Parsing route patterns into structured metadata (`parseRoutePattern`)
 * 3. Generating TypeScript interface definitions for API routes (`generateRoutesTypeDefinitions`)
 * 4. Route function registration (`get`, `post`, `route`) and metadata inspection
 */

import {
  del,
  generateRoutesTypeDefinitions,
  get,
  getRegisteredRoutesMeta,
  parseRoutePattern,
  post,
  Response,
  route,
  uri,
} from "../dist/main.mjs";

// 1. Type-safe URI construction
const userProfileUrl = uri("/users/:id/profile?<tab>", { id: "42", tab: "settings" });
console.log("Constructed URI:", userProfileUrl);
// Output: "/users/42/profile?tab=settings"

// 2. Parsing route patterns
const routeMeta = parseRoutePattern("/api/v1/posts/<id>/<action..>?<ref>&<verbose..>");
console.log("Parsed Route Metadata:", JSON.stringify(routeMeta, null, 2));

// 3. Automatic TypeScript interface generation for API routes
const patterns = ["/api/users/:id", "/api/posts/<category..>?<sort>", "/api/comments/:id"];
const tsDefinitions = generateRoutesTypeDefinitions(patterns);
console.log("\n--- Generated TypeScript Route Interfaces ---");
console.log(tsDefinitions);

// 4. Using route registration functions
get("/users/:id", () => new Response("User Profile"));
post("/users", () => Response.json({ created: true }, { status: 201 }));
del("/users/:id", () => new Response("User Deleted"));
route("PATCH", "/users/:id", () => new Response("User Patched"));

console.log("\nRegistered Route Metadata:");
console.log(getRegisteredRoutesMeta());

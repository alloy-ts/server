/**
 * Example: Using `@alloy-ts/server` Codegen Utilities & Type Generation
 *
 * Demonstrates:
 * 1. Type-safe URI construction (`uri`)
 * 2. Parsing route patterns into structured metadata (`parseRoutePattern`)
 * 3. Generating TypeScript interface definitions for API routes (`generateRoutesTypeDefinitions`)
 * 4. Route registration (`registerRoute`)
 */

import {
  buildRouteUri,
  generateRoutesTypeDefinitions,
  parseRoutePattern,
  registerRoute,
  Response,
  uri,
} from "../dist/index.js";

// 1. Type-safe URI construction
const userProfileUrl = uri("/users/:id/profile?<tab>", { id: "42", tab: "settings" });
console.log("Constructed URI:", userProfileUrl);

// 2. Parsing route patterns
const routeMeta = parseRoutePattern("/api/v1/posts/<id>/<action..>?<ref>&<verbose..>");
console.log("Parsed Route Metadata:", JSON.stringify(routeMeta, null, 2));

// 3. Automatic TypeScript interface generation for API routes
const patterns = ["/api/users/:id", "/api/posts/<category..>?<sort>", "/api/comments/:id"];
const tsDefinitions = generateRoutesTypeDefinitions(patterns);
console.log("\n--- Generated TypeScript Route Interfaces ---");
console.log(tsDefinitions);

// 4. Using route registration function
registerRoute("GET", "/users/:id", () => new Response("User Profile"));
registerRoute("POST", "/users", () => Response.json({ created: true }, { status: 201 }));

console.log("\nCodegen routes verified successfully.");

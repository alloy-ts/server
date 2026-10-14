/**
 * Code Generation (Codegen) Demo for Rocket NAPI Bindings
 *
 * Demonstrates:
 * - `parseRoute` and `parseRoutes` for extracting metadata from route specs
 * - `toRocketPath` for syntax translation
 * - `generateTypes` for route map TypeScript interface declarations
 * - `generateClient` for typed fetch SDKs
 * - `generateModule` for full standalone modules
 */

import {
  generateClient,
  generateModule,
  generateTypes,
  httpMethods,
  parseRoute,
  parseRoutes,
  toRocketPath,
} from "../dist/index.js";

console.log("=== Supported Rocket HTTP Methods ===");
console.log(httpMethods().join(", "));

console.log("\n=== Route Syntax Translation ===");
console.log("/users/:id/*  -> ", toRocketPath("/users/:id/*"));

console.log("\n=== Parse Single Route ===");
const route = parseRoute({
  method: "GET",
  path: "/users/:id/posts/*?<query>",
  response: "Post[]",
});
console.log("Key:        ", route.key);
console.log("RocketPath: ", route.rocketPath);
console.log("Handler Name:", route.name);
console.log("Params:     ", route.params);
console.log("Query:      ", route.query);

console.log("\n=== Parse & Deduplicate Routes ===");
const routes = parseRoutes([
  { method: "GET", path: "/users/:id", response: "{ id: string; name: string }" },
  { method: "POST", path: "/users", body: "{ name: string }", response: "{ id: string }" },
  { method: "GET", path: "/users/:id", response: "User" }, // duplicate path -> deduplicated name
]);

for (const r of routes) {
  console.log(`- ${r.key} => handler: ${r.name}`);
}

console.log("\n=== Generated Route Types (generateTypes) ===");
console.log(generateTypes(routes, { typesName: "AppApiRoutes" }));

console.log("\n=== Generated Client SDK (generateClient) ===");
console.log(
  generateClient(routes, {
    clientName: "UserSdk",
    baseUrl: "http://localhost:3000",
  }),
);

console.log("\n=== Generated Standalone Module (generateModule) ===");
console.log(
  generateModule(routes, {
    clientName: "UserApiClient",
    baseUrl: "http://localhost:3000",
  }),
);

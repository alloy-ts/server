import { uri, parseRoutePattern, generateRouteManifest } from "../src/main.ts";

console.log("Parsing route pattern /users/:id/posts/:postId:");
const patternInfo = parseRoutePattern("/users/:id/posts/:postId");
console.log(`  Raw: ${patternInfo.rawPattern}`);
console.log(`  Rocket: ${patternInfo.rocketPattern}`);
console.log(`  Path params: ${patternInfo.pathParams.join(", ")}`);

console.log("\nBuilding type-safe route URIs:");
const u1 = uri("/users/:id", { id: "42" });
console.log(`  uri("/users/:id", { id: 42 }) -> ${u1}`);

const u2 = uri("/search/:query", { query: "rocket web framework" });
console.log(`  uri("/search/:query", { query: "rocket web framework" }) -> ${u2}`);

console.log("\nGenerating route manifest:");
const manifest = generateRouteManifest([
  "/api/v1/health",
  "/users/:id",
  "/posts/:category/:slug",
  "/api/*",
]);

for (const route of manifest) {
  console.log(`  - ${route.rawPattern} -> ${route.rocketPattern} (wildcard: ${route.isWildcard})`);
}

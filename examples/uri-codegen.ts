import {
  buildRouteUri,
  generateRoutesTypeDefinitions,
  parseRoutePattern,
  uri,
} from "../src/main.ts";

console.log("=== 1. Type-Safe URI Generation ===");
const userUri = uri("/users/:id", { id: "42" });
console.log("uri('/users/:id', { id: '42' }) =>", userUri);

const searchUri = buildRouteUri("/search/<category>/<item..>?<page>&<sort>", {
  category: "electronics",
  item: "laptops/pro",
  page: 2,
  sort: "asc",
});
console.log("buildRouteUri(...) =>", searchUri);

console.log("\n=== 2. Route Pattern Parsing ===");
const routePattern = "/api/v1/posts/<id>/<comments..>?<ref>&<page>";
const parsed = parseRoutePattern(routePattern);
console.log(`Parsed pattern: ${routePattern}`);
console.log("Path parameters:", parsed.pathParams);
console.log("Query parameters:", parsed.queryParams);

console.log("\n=== 3. TypeScript Interface Codegen ===");
const routesToGenerate = [
  "/api/v1/health",
  "/api/v1/users/<id>",
  "/api/v1/posts/<id>/<comments..>?<ref>&<page>",
  "/api/v1/files/*",
];

const tsTypes = generateRoutesTypeDefinitions(routesToGenerate);
console.log(tsTypes);

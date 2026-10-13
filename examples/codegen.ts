/**
 * Rocket Code Generation (`codegen`) Example
 *
 * Demonstrates parsing route patterns, translating paths to Rocket syntax, and
 * generating TypeScript route maps, client SDKs, and full API modules using
 * `@alloy-ts/server`.
 */

import {
  generateClient,
  generateModule,
  generateTypes,
  parseRoute,
  parseRoutes,
  toRocketPath,
} from "@alloy-ts/server";

// 1. Translate route pattern to Rocket syntax
const rocketPath = toRocketPath("/users/:id/posts/*");
console.log("Translated Rocket path:", rocketPath);
// "/users/<id>/posts/<_..>"

// 2. Parse individual route pattern
const route = parseRoute({
  method: "GET",
  path: "/users/:id/posts/*?<page>&<limit>",
  response: "PostListResponse",
});

console.log("\nParsed Route Definition:");
console.log(`- Key: ${route.key}`);
console.log(`- Method: ${route.method}`);
console.log(`- Name: ${route.name}`);
console.log(`- Path params:`, route.params);
console.log(`- Query params:`, route.query);

// 3. Resolve route table with automatic name deduplication
const routes = parseRoutes([
  {
    method: "GET",
    path: "/users/:id",
    response: "User",
  },
  {
    method: "POST",
    path: "/users",
    body: "CreateUserPayload",
    response: "User",
  },
  {
    method: "DELETE",
    path: "/users/:id",
    response: "{ success: boolean }",
  },
]);

// 4. Generate TypeScript Route Types declaration
console.log("\n--- Generated Types ---");
const typesCode = generateTypes(routes, {
  typesName: "UserApiRouteTypes",
});
console.log(typesCode);

// 5. Generate typed fetch client SDK
console.log("--- Generated Client SDK ---");
const clientCode = generateClient(routes, {
  clientName: "UserApiClient",
  baseUrl: "https://api.example.com",
  typesImport: "./user-api-types",
});
console.log(clientCode);

// 6. Generate complete module (Routes + Types + URI Builder + Client)
console.log("--- Generated Complete Module ---");
const fullModuleCode = generateModule(routes, {
  clientName: "UserApiClient",
  baseUrl: "http://localhost:3000",
});
console.log(fullModuleCode.slice(0, 300) + "...\n[truncated]");

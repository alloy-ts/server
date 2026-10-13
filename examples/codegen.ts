import {
  generateClient,
  generateModule,
  generateTypes,
  parseRoutes,
  toRocketPath,
} from "../dist/index.js";

console.log("=== Rocket Route Syntax Translation ===");
console.log("Alloy -> Rocket path:", toRocketPath("/users/:id/posts/*"));

console.log("\n=== Parsing Route Table ===");
const routes = parseRoutes([
  {
    method: "GET",
    path: "/api/users/:id",
    response: "{ id: string; name: string; role: string }",
  },
  {
    method: "POST",
    path: "/api/users",
    body: "{ name: string; role: string }",
    response: "{ id: string; name: string; role: string }",
  },
  {
    method: "GET",
    path: "/api/posts/:category/*?<page>&<limit>",
    response: "Array<{ id: string; title: string }>",
  },
]);

console.log(`Parsed ${routes.length} routes.`);

console.log("\n=== Generated TypeScript Type Definitions ===");
const typeDefs = generateTypes(routes);
console.log(typeDefs);

console.log("\n=== Generated Fetch Client SDK ===");
const clientCode = generateClient(routes, {
  clientName: "UserApiClient",
  baseUrl: "http://localhost:3000",
});
console.log(clientCode);

console.log("\n=== Generated Full API Module ===");
const moduleCode = generateModule(routes, {
  clientName: "ApiClient",
  typesName: "ApiRouteMap",
});
console.log(`Module code generated (${moduleCode.length} characters).`);

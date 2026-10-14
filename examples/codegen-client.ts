import { generateClient, generateTypes, generateModule, parseRoutes } from "../src/main.ts";

// Resolve route specifications
const routes = parseRoutes([
  { method: "GET", path: "/users/:id", response: "User" },
  { method: "POST", path: "/users", body: "CreateUser", response: "User" },
  { method: "DELETE", path: "/users/:id", response: "{ success: boolean }" },
]);

// 1. Emit TypeScript interface declarations
const typesCode = generateTypes(routes);
console.log("=== Generated Route Types ===");
console.log(typesCode);

// 2. Emit strongly-typed fetch client SDK
const clientCode = generateClient(routes, {
  clientName: "UserApiClient",
  baseUrl: "http://localhost:3000",
});
console.log("=== Generated Client SDK ===");
console.log(clientCode);

// 3. Emit standalone API module
const moduleCode = generateModule(routes, {
  clientName: "Api",
  typesName: "ApiRoutes",
  baseUrl: "http://api.example.com",
});
console.log("=== Generated Standalone Module ===");
console.log(moduleCode);

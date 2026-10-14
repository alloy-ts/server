/**
 * Custom SDK Generation Example
 *
 * Demonstrates:
 * - Parsing multiple API route specifications
 * - Applying parameter type overrides (e.g., union types, numbers)
 * - Generating strongly-typed TypeScript interface declarations
 * - Generating a standalone fetch client SDK with custom options
 */

import { generateClient, generateTypes, parseRoutes } from "../dist/index.js";

const routes = parseRoutes([
  {
    method: "GET",
    path: "/api/v1/products/:id",
    response: "Product",
    paramTypes: [{ name: "id", tsType: "number" }],
  },
  {
    method: "GET",
    path: "/api/v1/products?<category>&<sort>",
    response: "Product[]",
    queryTypes: [
      { name: "category", tsType: '"electronics" | "books" | "clothing"' },
      { name: "sort", tsType: '"asc" | "desc"' },
    ],
  },
  {
    method: "POST",
    path: "/api/v1/products",
    body: "CreateProductInput",
    response: "Product",
  },
]);

console.log("=== Generated Type Declarations ===");
const typeDefs = generateTypes(routes, {
  typesName: "StoreApiRoutes",
  serverImport: "@alloy-ts/server",
});
console.log(typeDefs);

console.log("=== Generated Client SDK ===");
const clientSdk = generateClient(routes, {
  clientName: "StoreApiClient",
  baseUrl: "https://api.store.example.com",
});
console.log(clientSdk);

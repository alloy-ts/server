/**
 * TypeScript Code Generation (`codegen`) Example
 *
 * Demonstrates parsing route patterns, translating between Rocket and Alloy path
 * syntaxes, and emitting strongly-typed TypeScript clients and interfaces.
 */

import {
  generateRouteHandler,
  generateTypescriptClient,
  generateTypescriptInterface,
  parseRocketRoutePattern,
  parseRoutes,
  toRocketPath,
  translatePatternToAlloy,
} from "../src/main.ts";

console.log("=== Rocket Code Generation Example ===\n");

// 1. Route pattern parsing
const routePattern = "/users/<id>?<age>&<name..>";
const params = parseRocketRoutePattern(routePattern);
console.log(`Parsed parameters for "${routePattern}":`);
console.log(JSON.stringify(params, null, 2));

// 2. Route pattern translation
const alloyPath = translatePatternToAlloy("/users/<id>/<rest..>");
console.log(`\nRocket -> Alloy path: "/users/<id>/<rest..>" => "${alloyPath}"`);

const rocketPath = toRocketPath("/users/:id/*");
console.log(`Alloy -> Rocket path: "/users/:id/*" => "${rocketPath}"`);

// 3. Generate TypeScript Interface
const userInterface = generateTypescriptInterface("User", [
  { name: "id", tsType: "string", optional: false, description: "Unique user ID" },
  { name: "name", tsType: "string", optional: false, description: "User's full name" },
  { name: "email", tsType: "string", optional: true, description: "Optional email address" },
]);

console.log("\nGenerated TypeScript Interface:");
console.log(userInterface);

// 4. Generate Route Handler Boilerplate
const handlerCode = generateRouteHandler({ method: "GET", path: "/users/:id" });
console.log("Generated Route Handler:");
console.log(handlerCode);

// 5. Generate Strongly-Typed Fetch Client SDK
const clientResult = generateTypescriptClient(
  [
    {
      method: "GET",
      path: "/users/<id>",
      name: "getUser",
      returnType: "Promise<{ id: string; name: string; email?: string }>",
    },
    {
      method: "POST",
      path: "/users",
      name: "createUser",
      bodyType: "{ name: string; email?: string }",
      returnType: "Promise<{ id: string; name: string }>",
    },
  ],
  { clientName: "UserApiClient", baseUrl: "http://localhost:3000" },
);

console.log("Generated Client SDK:");
console.log(clientResult.code);

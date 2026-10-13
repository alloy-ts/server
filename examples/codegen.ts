import {
  generateClient,
  generateModule,
  generateTypes,
  parseRoutes,
  toRocketPath,
} from "../src/main.ts";

// 1. Parse route definitions
const routes = parseRoutes([
  { method: "GET", path: "/users/:id", response: "User" },
  { method: "POST", path: "/users", body: "CreateUser", response: "User" },
  { method: "PUT", path: "/users/:id", body: "UpdateUser", response: "User" },
  { method: "DELETE", path: "/users/:id", response: "{ success: boolean }" },
  { method: "GET", path: "/posts/*?limit", response: "Array<Post>" },
]);

console.log("Parsed Routes:");
for (const r of routes) {
  console.log(`- ${r.method} ${r.path} -> Rocket path: ${r.rocketPath} (Handler: ${r.name})`);
}

// 2. Pattern translation
const rocketPath = toRocketPath("/api/v1/users/:id/*");
console.log("\nPattern Translation:");
console.log("/api/v1/users/:id/* ->", rocketPath);

// 3. Generate TypeScript type definitions
console.log("\n--- Generated TypeScript Types ---");
console.log(generateTypes(routes));

// 4. Generate typed fetch Client SDK
console.log("\n--- Generated Typed Fetch Client ---");
console.log(generateClient(routes, { clientName: "UserApiClient" }));

// 5. Generate full API module (ROUTES constant + types + client)
console.log("\n--- Generated Complete API Module ---");
console.log(generateModule(routes, { baseUrl: "http://127.0.0.1:3000" }));

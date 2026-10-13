const assert = require("node:assert");
const {
  parseRoute,
  parseRoutes,
  parseRocketRoutePattern,
  translatePatternToAlloy,
  toRocketPath,
  generateTypes,
  generateClient,
  generateModule,
  generateTypescriptClient,
  generateRouteHandler,
  generateTypescriptInterface,
  httpMethods,
} = require("./dist/index.js");

console.log("Running NAPI binding tests in test.cjs...");

// httpMethods
const methods = httpMethods();
assert(Array.isArray(methods));
assert(methods.includes("GET"));
assert(methods.includes("POST"));

// parseRocketRoutePattern
const params = parseRocketRoutePattern("/users/<id>?<age>&<name..>");
assert.strictEqual(params.length, 3);
assert.strictEqual(params[0].name, "id");
assert.strictEqual(params[0].isPath, true);
assert.strictEqual(params[1].name, "age");
assert.strictEqual(params[1].isQuery, true);

// translatePatternToAlloy
const alloy = translatePatternToAlloy("/users/<id>/<rest..>");
assert.strictEqual(alloy, "/users/:id/*");

// toRocketPath
const rocketPath = toRocketPath("/users/:id/*");
assert.strictEqual(rocketPath, "/users/<id>/<_..>");

// parseRoute
const route = parseRoute({
  method: "GET",
  path: "/users/:id?<active>",
  response: "User",
});
assert.strictEqual(route.method, "GET");
assert.strictEqual(route.path, "/users/:id");
assert.strictEqual(route.name, "getUsersById");

// parseRoutes
const routes = parseRoutes([
  { method: "GET", path: "/users/:id", response: "User" },
  { method: "POST", path: "/users", body: "CreateUser", response: "User" },
]);
assert.strictEqual(routes.length, 2);

// generateTypes
const typesCode = generateTypes(routes);
assert(typesCode.includes("export interface RouteTypes"));

// generateClient
const clientCode = generateClient(routes);
assert(clientCode.includes("export class RocketClient"));

// generateModule
const moduleCode = generateModule(routes);
assert(moduleCode.includes("export const ROUTES"));

// generateTypescriptClient
const tsClientRes = generateTypescriptClient(
  [
    {
      method: "GET",
      path: "/users/<id>",
      name: "getUser",
      returnType: "Promise<{ id: string; name: string }>",
    },
  ],
  { clientName: "UserApiClient" },
);
assert(tsClientRes.code.includes("export class UserApiClient"));

// generateRouteHandler
const handlerCode = generateRouteHandler({ method: "GET", path: "/users/:id" });
assert(handlerCode.includes("getUsersByIdHandler"));

// generateTypescriptInterface
const ifaceCode = generateTypescriptInterface("User", [
  { name: "id", tsType: "string", optional: false },
]);
assert(ifaceCode.includes("export interface User"));

console.log("All CJS NAPI binding tests passed successfully!");

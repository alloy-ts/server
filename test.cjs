const assert = require("node:assert");

async function runTests() {
  console.log("Running NAPI binding and TS bundle tests in test.cjs...");

  const pkg = await import("./dist/main.mjs");

  const {
    App,
    Alloy,
    Response,
    Route,
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
  } = pkg;

  // httpMethods
  const methods = httpMethods();
  assert(Array.isArray(methods));
  assert(methods.includes("GET"));
  assert(methods.includes("POST"));

  // Route
  const routeIndex = new Route("GET", "/");
  assert.strictEqual(routeIndex.method, "GET");
  assert.strictEqual(routeIndex.uri, "/");
  assert.strictEqual(routeIndex.rank, -9);

  const rankedRoute = Route.ranked(2, "POST", "/foo?bar");
  assert.strictEqual(rankedRoute.method, "POST");
  assert.strictEqual(rankedRoute.uri, "/foo?bar");
  assert.strictEqual(rankedRoute.rank, 2);

  // App & Response
  const app = new App();
  assert(typeof app.get === "function");
  assert(typeof app.post === "function");

  const res = new Response("OK", { status: 200 });
  assert.strictEqual(res.status, 200);

  // Alloy.serve
  assert(typeof Alloy.serve === "function");

  // parseRocketRoutePattern
  const params = parseRocketRoutePattern("/users/<id>?<age>&<name..>");
  assert.strictEqual(params.length, 3);
  assert.strictEqual(params[0].name, "id");
  assert.strictEqual(params[0].isPath, true);

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

  console.log("All NAPI binding and TS bundle tests passed successfully!");
}

runTests().catch((err) => {
  console.error(err);
  process.exit(1);
});

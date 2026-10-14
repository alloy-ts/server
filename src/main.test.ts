import { expect, test } from "vite-plus/test";
import {
  App,
  Route,
  generateClient,
  generateModule,
  generateTypes,
  generateTypescriptClient,
  httpMethods,
  main,
  parseRocketRoutePattern,
  parseRoute,
  parseRoutes,
  response,
  toRocketPath,
  translatePatternToAlloy,
} from "./main.ts";

test("main returns Hello, world!", () => {
  expect(main()).toBe("Hello, world!");
});

test("httpMethods returns supported Rocket HTTP methods", () => {
  const methods = httpMethods();
  expect(methods).toContain("GET");
  expect(methods).toContain("POST");
  expect(methods).toContain("DELETE");
});

test("parseRocketRoutePattern extracts dynamic path and query parameters", () => {
  const params = parseRocketRoutePattern("/users/<id>?<age>&<name..>");
  expect(params).toHaveLength(3);

  expect(params[0]).toEqual({
    name: "id",
    paramType: "string",
    isPath: true,
    isQuery: false,
    isOptional: false,
  });

  expect(params[1]).toEqual({
    name: "age",
    paramType: "string",
    isPath: false,
    isQuery: true,
    isOptional: false,
  });

  expect(params[2]).toEqual({
    name: "name",
    paramType: "string",
    isPath: false,
    isQuery: true,
    isOptional: true,
  });
});

test("translatePatternToAlloy converts Rocket path to Alloy syntax", () => {
  expect(translatePatternToAlloy("/users/<id>/<rest..>")).toBe("/users/:id/*");
  expect(toRocketPath("/users/:id/*")).toBe("/users/<id>/<_..>");
});

test("parseRoute and parseRoutes generate resolved route definitions", () => {
  const route = parseRoute({
    method: "GET",
    path: "/users/:id",
    response: "User",
  });
  expect(route.method).toBe("GET");
  expect(route.path).toBe("/users/:id");
  expect(route.name).toBe("getUsersById");

  const routes = parseRoutes([
    { method: "GET", path: "/users/:id", response: "User" },
    { method: "POST", path: "/users", body: "CreateUser", response: "User" },
  ]);
  expect(routes).toHaveLength(2);
});

test("generateTypes and generateModule produce valid TypeScript output", () => {
  const routes = parseRoutes([{ method: "GET", path: "/users/:id", response: "User" }]);

  const typesOutput = generateTypes(routes);
  expect(typesOutput).toContain("export interface RouteTypes");
  expect(typesOutput).toContain('"GET /users/:id":');

  const moduleOutput = generateModule(routes);
  expect(moduleOutput).toContain("export const ROUTES =");
  expect(moduleOutput).toContain("export class RocketClient");

  const clientOutput = generateClient(routes);
  expect(clientOutput).toContain("export class RocketClient");
});

test("generateTypescriptClient produces strongly-typed fetch client SDK", () => {
  const result = generateTypescriptClient(
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

  expect(result.code).toContain("export class UserApiClient");
  expect(result.code).toContain("async getUser(");
});

test("App registers routes and launches server", () => {
  const app = new App();
  app.get("/", async () => response("Hello, world!"));
  app.post("/users", async () => response("Created", { status: 201 }));

  const server = app.launch({ port: 0 });
  expect(server.url).toContain("http://");

  app.stop();
});

test("Route computes default rank and handles explicit rank", () => {
  const index = new Route("GET", "/");
  expect(index.method).toBe("GET");
  expect(index.uri).toBe("/");
  expect(index.rank).toBe(-9);

  const staticQuery = new Route("GET", "/?foo");
  expect(staticQuery.rank).toBe(-12);

  const explicitRank = Route.ranked(2, "GET", "/route/<path..>?query");
  expect(explicitRank.method).toBe("GET");
  expect(explicitRank.uri).toBe("/route/<path..>?query");
  expect(explicitRank.rank).toBe(2);
});

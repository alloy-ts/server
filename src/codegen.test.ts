import { expect, test } from "vite-plus/test";
import {
  generateClient,
  generateModule,
  generateRouteHandler,
  generateTypes,
  generateTypescriptClient,
  generateTypescriptInterface,
  httpMethods,
  parseRocketRoutePattern,
  parseRoute,
  parseRoutes,
  toRocketPath,
  translatePatternToAlloy,
} from "../dist/index.js";

test("httpMethods returns all Rocket supported HTTP methods", () => {
  const methods = httpMethods();
  expect(methods).toEqual([
    "GET",
    "PUT",
    "POST",
    "DELETE",
    "OPTIONS",
    "HEAD",
    "PATCH",
    "TRACE",
    "CONNECT",
  ]);
});

test("parseRocketRoutePattern parses path and query parameters", () => {
  const params = parseRocketRoutePattern("/items/<id>/<category..>?<sort>&<filter>");
  expect(params).toHaveLength(4);

  expect(params[0]).toEqual({
    name: "id",
    paramType: "string",
    isPath: true,
    isQuery: false,
    isOptional: false,
  });

  expect(params[1]).toEqual({
    name: "category",
    paramType: "string",
    isPath: true,
    isQuery: false,
    isOptional: true,
  });

  expect(params[2]).toEqual({
    name: "sort",
    paramType: "string",
    isPath: false,
    isQuery: true,
    isOptional: false,
  });

  expect(params[3]).toEqual({
    name: "filter",
    paramType: "string",
    isPath: false,
    isQuery: true,
    isOptional: false,
  });
});

test("translatePatternToAlloy and toRocketPath bidirectionally convert route syntax", () => {
  expect(translatePatternToAlloy("/api/v1/<user>/<rest..>")).toBe("/api/v1/:user/*");
  expect(toRocketPath("/api/v1/:user/*")).toBe("/api/v1/<user>/<_..>");
});

test("parseRoute resolves single route input with type overrides and custom name", () => {
  const route = parseRoute({
    method: "GET",
    path: "/users/:id?page",
    name: "fetchUser",
    response: "UserResponse",
    paramTypes: [{ name: "id", tsType: "number" }],
    queryTypes: [{ name: "page", tsType: "number" }],
  });

  expect(route.key).toBe("GET /users/:id");
  expect(route.method).toBe("GET");
  expect(route.path).toBe("/users/:id");
  expect(route.rocketPath).toBe("/users/<id>");
  expect(route.name).toBe("fetchUser");
  expect(route.response).toBe("UserResponse");

  expect(route.params).toHaveLength(1);
  expect(route.params[0].name).toBe("id");
  expect(route.params[0].tsType).toBe("number");

  expect(route.query).toHaveLength(1);
  expect(route.query[0].name).toBe("page");
  expect(route.query[0].tsType).toBe("number");
});

test("parseRoutes resolves route array and deduplicates names", () => {
  const routes = parseRoutes([
    { method: "POST", path: "/posts", body: "CreatePost", response: "Post" },
    { method: "POST", path: "/posts", body: "CreatePost", response: "Post" },
  ]);

  expect(routes).toHaveLength(2);
  expect(routes[0].name).toBe("postPosts");
  expect(routes[1].name).toBe("postPosts_2");
});

test("generateTypes, generateClient, and generateModule emit valid TypeScript source", () => {
  const routes = parseRoutes([
    { method: "GET", path: "/users/:id", response: "User" },
    { method: "POST", path: "/users", body: "CreateUser", response: "User" },
  ]);

  const typesCode = generateTypes(routes, { typesName: "MyRouteTypes" });
  expect(typesCode).toContain("export interface MyRouteTypes {");
  expect(typesCode).toContain('"GET /users/:id":');
  expect(typesCode).toContain('"POST /users":');

  const clientCode = generateClient(routes, {
    clientName: "MyApiClient",
    baseUrl: "https://api.example.com",
  });
  expect(clientCode).toContain("export class MyApiClient {");
  expect(clientCode).toContain("https://api.example.com");
  expect(clientCode).toContain("async getUsersById(");
  expect(clientCode).toContain("async postUsers(");

  const moduleCode = generateModule(routes, {
    clientName: "FullClient",
    typesName: "FullTypes",
  });
  expect(moduleCode).toContain("export const ROUTES =");
  expect(moduleCode).toContain("export interface FullTypes");
  expect(moduleCode).toContain("export class FullClient");
  expect(moduleCode).toContain("export function uri(");
});

test("generateTypescriptClient generates client SDK from RouteSpecInput list", () => {
  const result = generateTypescriptClient(
    [
      {
        method: "GET",
        path: "/products/<id>",
        name: "getProduct",
        returnType: "Product",
      },
    ],
    { clientName: "ProductStoreClient" },
  );

  expect(result.code).toContain("export class ProductStoreClient");
  expect(result.code).toContain("async getProduct(");
  expect(result.code).toContain("as ResponseOf<\"GET /products/:id\">");
});

test("generateRouteHandler generates boilerplate handler function", () => {
  const handlerCode = generateRouteHandler({
    method: "DELETE",
    path: "/items/:id",
    name: "deleteItem",
  });

  expect(handlerCode).toContain(
    'export const deleteItemHandler: RouteHandler<"DELETE /items/:id"> = async (ctx) => {',
  );
  expect(handlerCode).toContain("// TODO: implement DELETE handler for /items/:id");
  expect(handlerCode).toContain('return new Response("OK");');
});

test("generateTypescriptInterface formats field definitions into TypeScript interface", () => {
  const interfaceCode = generateTypescriptInterface("UserProfile", [
    { name: "id", tsType: "string", optional: false, description: "Unique user identifier" },
    { name: "nickname", tsType: "string", optional: true, description: "Display name" },
    { name: "age", tsType: "number", optional: true },
  ]);

  expect(interfaceCode).toContain("export interface UserProfile {");
  expect(interfaceCode).toContain("/** Unique user identifier */");
  expect(interfaceCode).toContain("id: string;");
  expect(interfaceCode).toContain("/** Display name */");
  expect(interfaceCode).toContain("nickname?: string;");
  expect(interfaceCode).toContain("age?: number;");
});

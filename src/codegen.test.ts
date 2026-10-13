import { expect, test } from "vite-plus/test";
import {
  httpMethods,
  parseRoute,
  parseRoutes,
  toRocketPath,
  generateTypes,
  generateClient,
  generateModule,
} from "../dist/index.js";

test("httpMethods returns supported HTTP methods", () => {
  const methods = httpMethods();
  expect(methods).toContain("GET");
  expect(methods).toContain("POST");
  expect(methods).toContain("DELETE");
  expect(methods).toContain("PATCH");
  expect(methods).not.toContain("FETCH");
});

test("toRocketPath converts Express/Alloy path patterns to Rocket syntax", () => {
  expect(toRocketPath("/users/:id/posts/*")).toBe("/users/<id>/posts/<_..>");
  expect(toRocketPath("/files/<path..>")).toBe("/files/<path..>");
  expect(toRocketPath("/")).toBe("/");
});

test("parseRoute parses route inputs and extracts parameters", () => {
  const route = parseRoute({
    method: "get",
    path: "/users/:id/posts/*?<sort>",
    response: "Post[]",
    paramTypes: [{ name: "id", tsType: "number" }],
  });

  expect(route.method).toBe("GET");
  expect(route.path).toBe("/users/:id/posts/*");
  expect(route.rocketPath).toBe("/users/<id>/posts/<_..>");
  expect(route.key).toBe("GET /users/:id/posts/*");
  expect(route.name).toBe("getUsersByIdPosts");
  expect(route.response).toBe("Post[]");
  expect(route.params.length).toBe(2);
  expect(route.params[0]?.name).toBe("id");
  expect(route.params[0]?.tsType).toBe("number");
  expect(route.params[1]?.name).toBe("_");
  expect(route.params[1]?.trailing).toBe(true);
  expect(route.query.length).toBe(1);
  expect(route.query[0]?.name).toBe("sort");
});

test("parseRoutes parses multiple route definitions and dedupes handler names", () => {
  const routes = parseRoutes([
    { method: "GET", path: "/items/:id" },
    { method: "GET", path: "/items/:id" },
  ]);

  expect(routes.length).toBe(2);
  expect(routes[0]?.name).toBe("getItemsById");
  expect(routes[1]?.name).toBe("getItemsById_2");
});

test("generateTypes emits TypeScript type definitions", () => {
  const routes = parseRoutes([
    {
      method: "GET",
      path: "/users/:id",
      response: "User",
    },
    {
      method: "POST",
      path: "/users",
      body: "CreateUser",
      response: "User",
    },
  ]);

  const types = generateTypes(routes, { typesName: "ApiRoutes" });
  expect(types).toContain("export interface ApiRoutes {");
  expect(types).toContain('"GET /users/:id": {');
  expect(types).toContain('"POST /users": {');
  expect(types).toContain("response: User;");
  expect(types).toContain("body: CreateUser;");
  expect(types).toContain("export type ParamsOf<K extends RouteKey>");
});

test("generateClient emits typed fetch client SDK", () => {
  const routes = parseRoutes([
    {
      method: "GET",
      path: "/users/:id",
      response: "User",
    },
  ]);

  const client = generateClient(routes, {
    clientName: "UserClient",
    baseUrl: "http://localhost:3000",
  });
  expect(client).toContain("export class UserClient {");
  expect(client).toContain("http://localhost:3000");
  expect(client).toContain("async getUsersById(");
  expect(client).toContain('as ResponseOf<"GET /users/:id">;');
});

test("generateModule emits full TypeScript module including routes, types, uri helper, and client", () => {
  const routes = parseRoutes([
    {
      method: "GET",
      path: "/status",
      response: "{ status: string }",
    },
  ]);

  const moduleCode = generateModule(routes, { clientName: "StatusClient" });
  expect(moduleCode).toContain("export const ROUTES = {");
  expect(moduleCode).toContain('"GET /status": "/status",');
  expect(moduleCode).toContain("export interface RouteTypes {");
  expect(moduleCode).toContain("export function uri(");
  expect(moduleCode).toContain("export class StatusClient {");
});

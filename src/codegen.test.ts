import { expect, test } from "vite-plus/test";
import {
  generateClient,
  generateModule,
  generateTypes,
  httpMethods,
  parseRoute,
  parseRoutes,
  toRocketPath,
} from "../dist/index.js";

test("httpMethods returns valid HTTP methods", () => {
  const methods = httpMethods();
  expect(methods).toContain("GET");
  expect(methods).toContain("POST");
  expect(methods).toContain("PUT");
  expect(methods).toContain("DELETE");
  expect(methods).toContain("PATCH");
  expect(methods).not.toContain("FETCH");
});

test("toRocketPath translates route patterns to Rocket format", () => {
  expect(toRocketPath("/users/:id/*")).toBe("/users/<id>/<_..>");
  expect(toRocketPath("/posts/<id..>")).toBe("/posts/<id..>");
  expect(toRocketPath("/")).toBe("/");
});

test("parseRoute parses Bun-style and Rocket-style routes", () => {
  const bunRoute = parseRoute({
    method: "get",
    path: "/users/:id/posts/*",
  });

  expect(bunRoute.method).toBe("GET");
  expect(bunRoute.path).toBe("/users/:id/posts/*");
  expect(bunRoute.rocketPath).toBe("/users/<id>/posts/<_..>");
  expect(bunRoute.key).toBe("GET /users/:id/posts/*");
  expect(bunRoute.name).toBe("getUsersByIdPosts");
  expect(bunRoute.params).toHaveLength(2);

  expect(bunRoute.params[0]).toEqual({
    name: "id",
    tsName: "id",
    kind: "Path",
    tsType: "string | number",
    trailing: false,
    optional: false,
  });

  expect(bunRoute.params[1]).toEqual({
    name: "_",
    tsName: "_",
    kind: "Path",
    tsType: "string[]",
    trailing: true,
    optional: true,
  });

  const rocketRoute = parseRoute({
    method: "POST",
    path: "/users/<id..>?<q>&closed",
  });

  expect(rocketRoute.path).toBe("/users/:id");
  expect(rocketRoute.rocketPath).toBe("/users/<id..>");
  expect(rocketRoute.params[0]?.trailing).toBe(true);
  expect(rocketRoute.query).toHaveLength(2);
  expect(rocketRoute.query[0]?.name).toBe("q");
  expect(rocketRoute.query[1]?.name).toBe("closed");
});

test("parseRoute handles reserved keywords and type overrides", () => {
  const route = parseRoute({
    method: "GET",
    path: "/items/:type?2fa",
    paramTypes: [{ name: "type", tsType: '"a" | "b"' }],
  });

  expect(route.params[0]?.tsName).toBe("type_");
  expect(route.params[0]?.tsType).toBe('"a" | "b"');
  expect(route.query[0]?.tsName).toBe("param0_2fa");
});

test("parseRoute throws error on invalid HTTP method", () => {
  expect(() =>
    parseRoute({
      method: "INVALID_METHOD",
      path: "/test",
    }),
  ).toThrow(/invalid HTTP method/);
});

test("parseRoutes deduplicates generated route names", () => {
  const defs = parseRoutes([
    { method: "GET", path: "/a/:id" },
    { method: "GET", path: "/a/:id" },
  ]);

  expect(defs[0]?.name).toBe("getAById");
  expect(defs[1]?.name).toBe("getAById_2");
});

test("generateTypes emits type declarations", () => {
  const route = parseRoute({
    method: "GET",
    path: "/users/:id?<active>",
    response: "User",
  });

  const code = generateTypes([route]);

  expect(code).toContain("export interface RouteTypes {");
  expect(code).toContain('"GET /users/:id": {');
  expect(code).toContain("id: string | number;");
  expect(code).toContain("active?: string | number;");
  expect(code).toContain("response: User;");
  expect(code).toContain("export type ParamsOf<K extends RouteKey>");
  expect(code).toContain("export interface RouteContext<K extends RouteKey>");
  expect(code).not.toContain("export class");
  expect(code).not.toContain("export function uri(");
});

test("generateClient emits typed client code", () => {
  const route = parseRoute({
    method: "GET",
    path: "/users/:id?<active>",
    response: "User",
  });

  const clientCode = generateClient([route], {
    clientName: "UserClient",
    baseUrl: "https://api.example.com",
    typesImport: "./types",
  });

  expect(clientCode).toContain(
    'import type { ParamsOf, QueryOf, BodyOf, ResponseOf } from "./types";',
  );
  expect(clientCode).toContain("export class UserClient {");
  expect(clientCode).toContain("export function uri(");
  expect(clientCode).toContain("https://api.example.com");
  expect(clientCode).toContain("async getUsersById(");
});

test("generateModule emits full TypeScript module", () => {
  const route = parseRoute({
    method: "POST",
    path: "/users",
    body: "CreateUser",
    response: "User",
  });

  const moduleCode = generateModule([route]);

  expect(moduleCode).toContain("export const ROUTES = {");
  expect(moduleCode).toContain('"POST /users": "/users",');
  expect(moduleCode).toContain("export interface RouteTypes {");
  expect(moduleCode).toContain("export function uri(");
  expect(moduleCode).toContain("export class RocketClient {");
  expect(moduleCode).toContain("async postUsers(");
});

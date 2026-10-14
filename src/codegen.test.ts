import { expect, test } from "vite-plus/test";
import {
  parseRoute,
  parseRoutes,
  toRocketPath,
  httpMethods,
  generateTypes,
  generateClient,
  generateModule,
} from "../dist/index.js";

test("httpMethods returns supported HTTP method names", () => {
  const methods = httpMethods();
  expect(methods).toContain("GET");
  expect(methods).toContain("POST");
  expect(methods).toContain("DELETE");
});

test("toRocketPath translates paths correctly", () => {
  expect(toRocketPath("/users/:id")).toBe("/users/<id>");
  expect(toRocketPath("/static/*")).toBe("/static/<_..>");
});

test("parseRoute and parseRoutes generate route definitions", () => {
  const route = parseRoute({ method: "GET", path: "/users/:id" });
  expect(route.method).toBe("GET");
  expect(route.path).toBe("/users/:id");
  expect(route.rocketPath).toBe("/users/<id>");

  const routes = parseRoutes([
    { method: "GET", path: "/items/:id" },
    { method: "GET", path: "/items/:id" },
  ]);
  expect(routes.length).toBe(2);
  expect(routes[0]?.name).toBe("getItemsById");
  expect(routes[1]?.name).toBe("getItemsById_2");
});

test("generateTypes, generateClient, and generateModule emit TypeScript code", () => {
  const routes = parseRoutes([
    { method: "GET", path: "/api/users/:id", name: "getUser", response: "User" },
  ]);

  const types = generateTypes(routes);
  expect(types).toContain("export interface RouteTypes {");
  expect(types).toContain('"GET /api/users/:id": {');

  const client = generateClient(routes, { clientName: "UserClient" });
  expect(client).toContain("export class UserClient {");
  expect(client).toContain("async getUser(");

  const mod = generateModule(routes);
  expect(mod).toContain("export const ROUTES = {");
  expect(mod).toContain("export class RocketClient {");
});

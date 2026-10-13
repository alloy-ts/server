import { describe, expect, test } from "vite-plus/test";
import {
  generateClient,
  generateModule,
  generateTypes,
  httpMethods,
  parseRoute,
  parseRoutes,
  toRocketPath,
} from "../dist/index.js";

describe("httpMethods", () => {
  test("returns supported HTTP methods", () => {
    const methods = httpMethods();
    expect(methods).toContain("GET");
    expect(methods).toContain("POST");
    expect(methods).toContain("PUT");
    expect(methods).toContain("DELETE");
    expect(methods).toContain("PATCH");
    expect(methods).not.toContain("FETCH");
  });
});

describe("toRocketPath", () => {
  test("translates Alloy/Bun pattern to Rocket pattern", () => {
    expect(toRocketPath("/users/:id")).toBe("/users/<id>");
    expect(toRocketPath("/files/*")).toBe("/files/<_..>");
    expect(toRocketPath("/a/:b/*")).toBe("/a/<b>/<_..>");
    expect(toRocketPath("/a/<b..>")).toBe("/a/<b..>");
  });
});

describe("parseRoute & parseRoutes", () => {
  test("parses a simple path with dynamic params and query", () => {
    const route = parseRoute({
      method: "GET",
      path: "/users/:id?<active>",
      response: "User",
    });

    expect(route.method).toBe("GET");
    expect(route.path).toBe("/users/:id");
    expect(route.rocketPath).toBe("/users/<id>");
    expect(route.key).toBe("GET /users/:id");
    expect(route.name).toBe("getUsersById");
    expect(route.response).toBe("User");

    expect(route.params).toHaveLength(1);
    expect(route.params[0]).toMatchObject({
      name: "id",
      tsName: "id",
      kind: "Path",
      tsType: "string | number",
      trailing: false,
      optional: false,
    });

    expect(route.query).toHaveLength(1);
    expect(route.query[0]).toMatchObject({
      name: "active",
      tsName: "active",
      kind: "Query",
      tsType: "string | number",
      optional: true,
    });
  });

  test("handles type overrides and reserved identifiers", () => {
    const route = parseRoute({
      method: "GET",
      path: "/items/:type?2fa",
      paramTypes: [{ name: "type", tsType: '"a" | "b"' }],
    });

    expect(route.params[0]!.tsName).toBe("type_");
    expect(route.params[0]!.tsType).toBe('"a" | "b"');
    expect(route.query[0]!.tsName).toBe("param0_2fa");
  });

  test("deduplicates route handler names in parseRoutes", () => {
    const routes = parseRoutes([
      { method: "GET", path: "/a/:id" },
      { method: "GET", path: "/a/:id" },
    ]);

    expect(routes[0]!.name).toBe("getAById");
    expect(routes[1]!.name).toBe("getAById_2");
  });

  test("throws on invalid HTTP method", () => {
    expect(() => parseRoute({ method: "INVALID", path: "/" })).toThrow(/invalid HTTP method/i);
  });
});

describe("codegen emission", () => {
  const sampleRoute = parseRoute({
    method: "GET",
    path: "/users/:id?<active>",
    response: "User",
  });

  test("generateTypes emits type declarations", () => {
    const typesCode = generateTypes([sampleRoute]);

    expect(typesCode).toContain("export interface RouteTypes {");
    expect(typesCode).toContain('"GET /users/:id": {');
    expect(typesCode).toContain("id: string | number;");
    expect(typesCode).toContain("active?: string | number;");
    expect(typesCode).toContain("response: User;");
    expect(typesCode).toContain("export type ParamsOf<K extends RouteKey>");
    expect(typesCode).not.toContain("export class RocketClient");
  });

  test("generateClient emits typed client class and uri helper", () => {
    const clientCode = generateClient([sampleRoute], {
      clientName: "UserApiClient",
      baseUrl: "http://localhost:3000",
    });

    expect(clientCode).toContain("export class UserApiClient {");
    expect(clientCode).toContain("export function uri(");
    expect(clientCode).toContain("http://localhost:3000");
    expect(clientCode).toContain("async getUsersById(");
  });

  test("generateModule emits complete module", () => {
    const moduleCode = generateModule([sampleRoute]);

    expect(moduleCode).toContain("export const ROUTES = {");
    expect(moduleCode).toContain("export interface RouteTypes {");
    expect(moduleCode).toContain("export function uri(");
    expect(moduleCode).toContain("export class RocketClient {");
  });
});

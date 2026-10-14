import { describe, expect, it } from "vite-plus/test";
import {
  parseRoute,
  parseRoutes,
  toRocketPath,
  httpMethods,
  generateTypes,
  generateClient,
  generateModule,
} from "../dist/index.js";

describe("Rocket Codegen NAPI bindings", () => {
  it("lists supported HTTP methods", () => {
    const methods = httpMethods();
    expect(methods).toContain("GET");
    expect(methods).toContain("POST");
    expect(methods).toContain("PUT");
    expect(methods).toContain("DELETE");
    expect(methods).toContain("OPTIONS");
    expect(methods).toContain("HEAD");
    expect(methods).toContain("PATCH");
  });

  it("translates Bun/Alloy path syntax to Rocket syntax", () => {
    expect(toRocketPath("/users/:id")).toBe("/users/<id>");
    expect(toRocketPath("/posts/*")).toBe("/posts/<_..>");
  });

  it("parses a single route input into RouteDef", () => {
    const def = parseRoute({
      method: "GET",
      path: "/users/:id",
      response: "User",
    });

    expect(def.method).toBe("GET");
    expect(def.path).toBe("/users/:id");
    expect(def.rocketPath).toBe("/users/<id>");
    expect(def.key).toBe("GET /users/:id");
    expect(def.name).toBe("getUsersById");
    expect(def.params).toHaveLength(1);
    expect(def.params[0]?.name).toBe("id");
    expect(def.params[0]?.tsName).toBe("id");
    expect(def.response).toBe("User");
  });

  it("parses a list of routes and dedupes names", () => {
    const defs = parseRoutes([
      { method: "GET", path: "/users/:id" },
      { method: "GET", path: "/users/:id" },
    ]);

    expect(defs).toHaveLength(2);
    expect(defs[0]?.name).toBe("getUsersById");
    expect(defs[1]?.name).toBe("getUsersById_2");
  });

  it("generates TypeScript types", () => {
    const def = parseRoute({
      method: "GET",
      path: "/users/:id",
      response: "User",
    });

    const types = generateTypes([def]);
    expect(types).toContain("export interface RouteTypes {");
    expect(types).toContain('"GET /users/:id": {');
    expect(types).toContain("params: {\n      id: string | number;\n    };");
    expect(types).toContain("response: User;");
  });

  it("generates TypeScript fetch client", () => {
    const def = parseRoute({
      method: "POST",
      path: "/users",
      body: "CreateUser",
      response: "User",
    });

    const client = generateClient([def], {
      clientName: "ApiClient",
      baseUrl: "http://api.example.com",
    });

    expect(client).toContain("export class ApiClient {");
    expect(client).toContain("async postUsers(");
    expect(client).toContain("http://api.example.com");
  });

  it("generates a complete TypeScript module", () => {
    const def = parseRoute({
      method: "GET",
      path: "/health",
      response: "HealthStatus",
    });

    const mod = generateModule([def]);
    expect(mod).toContain("export const ROUTES = {");
    expect(mod).toContain('"GET /health": "/health",');
    expect(mod).toContain("export interface RouteTypes {");
    expect(mod).toContain("export function uri(");
    expect(mod).toContain("export class RocketClient {");
  });
});

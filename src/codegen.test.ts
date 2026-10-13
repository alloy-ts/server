import { describe, expect, it } from "vite-plus/test";
import {
  generateClient,
  generateModule,
  generateTypes,
  httpMethods,
  ParamKind,
  parseRoute,
  parseRoutes,
  toRocketPath,
} from "../dist/index.js";

describe("codegen", () => {
  describe("httpMethods", () => {
    it("returns supported Rocket HTTP methods", () => {
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
      expect(methods).toContain("GET");
      expect(methods).toContain("POST");
      expect(methods).not.toContain("FETCH");
    });
  });

  describe("toRocketPath", () => {
    it("translates Alloy/Bun pattern syntax to Rocket syntax", () => {
      expect(toRocketPath("/users/:id")).toBe("/users/<id>");
      expect(toRocketPath("/users/:id/posts/*")).toBe("/users/<id>/posts/<_..>");
      expect(toRocketPath("/files/*")).toBe("/files/<_..>");
      expect(toRocketPath("/users/<id..>")).toBe("/users/<id..>");
    });
  });

  describe("parseRoute", () => {
    it("parses Bun-style route pattern", () => {
      const def = parseRoute({
        method: "get",
        path: "/users/:id/posts/*",
      });

      expect(def.method).toBe("GET");
      expect(def.path).toBe("/users/:id/posts/*");
      expect(def.rocketPath).toBe("/users/<id>/posts/<_..>");
      expect(def.key).toBe("GET /users/:id/posts/*");
      expect(def.name).toBe("getUsersByIdPosts");
      expect(def.params.length).toBe(2);

      expect(def.params[0]).toEqual({
        name: "id",
        tsName: "id",
        kind: ParamKind.Path,
        tsType: "string | number",
        trailing: false,
        optional: false,
      });

      expect(def.params[1]).toEqual({
        name: "_",
        tsName: "_",
        kind: ParamKind.Path,
        tsType: "string[]",
        trailing: true,
        optional: true,
      });
    });

    it("parses Rocket-style route pattern with query parameters", () => {
      const def = parseRoute({
        method: "POST",
        path: "/users/<id..>?<q>&<active>",
      });

      expect(def.method).toBe("POST");
      expect(def.path).toBe("/users/:id");
      expect(def.rocketPath).toBe("/users/<id..>");
      expect(def.key).toBe("POST /users/:id");
      expect(def.params[0]?.trailing).toBe(true);

      expect(def.query.length).toBe(2);
      expect(def.query[0]?.name).toBe("q");
      expect(def.query[1]?.name).toBe("active");
      expect(def.query.every((q) => q.optional)).toBe(true);
    });

    it("handles parameter type overrides and reserved words", () => {
      const def = parseRoute({
        method: "GET",
        path: "/items/:type?2fa",
        paramTypes: [{ name: "type", tsType: '"a" | "b"' }],
      });

      // 'type' is a TS reserved word, so tsName should be sanitized to 'type_'
      expect(def.params[0]?.name).toBe("type");
      expect(def.params[0]?.tsName).toBe("type_");
      expect(def.params[0]?.tsType).toBe('"a" | "b"');

      // Invalid identifier '2fa' in query is sanitized
      expect(def.query[0]?.name).toBe("param0_2fa");
      expect(def.query[0]?.tsName).toBe("param0_2fa");
    });

    it("throws error for unsupported HTTP methods", () => {
      expect(() =>
        parseRoute({
          method: "INVALID",
          path: "/test",
        }),
      ).toThrow("invalid HTTP method `INVALID`");
    });
  });

  describe("parseRoutes", () => {
    it("deduplicates generated handler names for colliding routes", () => {
      const defs = parseRoutes([
        { method: "GET", path: "/a/:id" },
        { method: "GET", path: "/a/:id" },
        { method: "GET", path: "/a/:id" },
      ]);

      expect(defs[0]?.name).toBe("getAById");
      expect(defs[1]?.name).toBe("getAById_2");
      expect(defs[2]?.name).toBe("getAById_3");
    });
  });

  describe("generateTypes", () => {
    it("emits TypeScript interface and type definitions", () => {
      const def = parseRoute({
        method: "GET",
        path: "/users/:id?<active>",
        response: "User",
      });

      const types = generateTypes([def]);
      expect(types).toContain("export interface RouteTypes {");
      expect(types).toContain('"GET /users/:id": {');
      expect(types).toContain("id: string | number;");
      expect(types).toContain("active?: string | number;");
      expect(types).toContain("response: User;");
      expect(types).toContain("export type ParamsOf<K extends RouteKey>");
      expect(types).toContain("export type RouteHandler<K extends RouteKey>");
      expect(types).not.toContain("export class");
      expect(types).not.toContain("export function uri(");
    });

    it("respects options like custom typesName and serverImport", () => {
      const def = parseRoute({ method: "GET", path: "/health" });
      const types = generateTypes([def], {
        typesName: "AppRoutes",
        serverImport: "@alloy-ts/server",
        banner: false,
      });

      expect(types).toContain("export interface AppRoutes {");
      expect(types).toContain('import type { RequestInput as RequestLike } from "@alloy-ts/server";');
      expect(types).not.toContain("DO NOT EDIT");
    });
  });

  describe("generateClient", () => {
    it("emits typed fetch client class and uri helper", () => {
      const def = parseRoute({
        method: "GET",
        path: "/users/:id",
        response: "User",
      });

      const client = generateClient([def]);
      expect(client).toContain("export class RocketClient {");
      expect(client).toContain("export function uri(");
      expect(client).toContain("async getUsersById(params: ParamsOf<\"GET /users/:id\">, init?: RequestInit): Promise<ResponseOf<\"GET /users/:id\">>");
    });

    it("supports custom clientName, baseUrl, and typesImport", () => {
      const def = parseRoute({ method: "POST", path: "/items", body: "CreateItem", response: "Item" });
      const client = generateClient([def], {
        clientName: "ApiClient",
        baseUrl: "https://api.example.com",
        typesImport: "./types.js",
      });

      expect(client).toContain("import type { ParamsOf, QueryOf, BodyOf, ResponseOf } from \"./types.js\";");
      expect(client).toContain("export class ApiClient {");
      expect(client).toContain("https://api.example.com");
    });
  });

  describe("generateModule", () => {
    it("emits full module including ROUTES, types, uri helper, and client", () => {
      const def = parseRoute({
        method: "GET",
        path: "/users/:id",
        response: "User",
      });

      const code = generateModule([def]);
      expect(code).toContain("export const ROUTES = {");
      expect(code).toContain('"GET /users/:id": "/users/:id",');
      expect(code).toContain("export interface RouteTypes {");
      expect(code).toContain("export function uri(");
      expect(code).toContain("export class RocketClient {");
    });
  });
});

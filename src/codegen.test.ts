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

describe("Rocket Codegen NAPI Bindings", () => {
  describe("httpMethods", () => {
    test("returns expected HTTP methods supported by Rocket", () => {
      const methods = httpMethods();
      expect(methods).toContain("GET");
      expect(methods).toContain("POST");
      expect(methods).toContain("PUT");
      expect(methods).toContain("DELETE");
      expect(methods).toContain("OPTIONS");
      expect(methods).toContain("HEAD");
      expect(methods).toContain("PATCH");
      expect(methods).toContain("TRACE");
      expect(methods).toContain("CONNECT");
      expect(methods).not.toContain("FETCH");
    });
  });

  describe("toRocketPath", () => {
    test("translates Bun/Alloy path syntax to Rocket path syntax", () => {
      expect(toRocketPath("/users/:id")).toBe("/users/<id>");
      expect(toRocketPath("/files/*")).toBe("/files/<_..>");
      expect(toRocketPath("/a/<b..>")).toBe("/a/<b..>");
      expect(toRocketPath("/")).toBe("/");
    });
  });

  describe("parseRoute", () => {
    test("parses a standard route pattern and derives route metadata", () => {
      const route = parseRoute({
        method: "get",
        path: "/users/:id/posts/*",
      });

      expect(route.method).toBe("GET");
      expect(route.path).toBe("/users/:id/posts/*");
      expect(route.rocketPath).toBe("/users/<id>/posts/<_..>");
      expect(route.key).toBe("GET /users/:id/posts/*");
      expect(route.name).toBe("getUsersByIdPosts");
      expect(route.params.length).toBe(2);

      expect(route.params[0]).toEqual({
        name: "id",
        tsName: "id",
        kind: "Path",
        tsType: "string | number",
        trailing: false,
        optional: false,
      });

      expect(route.params[1]).toEqual({
        name: "_",
        tsName: "_",
        kind: "Path",
        tsType: "string[]",
        trailing: true,
        optional: true,
      });
    });

    test("parses Rocket-style path and query params", () => {
      const route = parseRoute({
        method: "POST",
        path: "/users/<id..>?<q>&closed",
      });

      expect(route.path).toBe("/users/:id");
      expect(route.rocketPath).toBe("/users/<id..>");
      expect(route.key).toBe("POST /users/:id");
      expect(route.params[0]!.trailing).toBe(true);

      expect(route.query.length).toBe(2);
      expect(route.query[0]).toEqual({
        name: "q",
        tsName: "q",
        kind: "Query",
        tsType: "string | number",
        trailing: false,
        optional: true,
      });
      expect(route.query[1]).toEqual({
        name: "closed",
        tsName: "closed",
        kind: "Query",
        tsType: "string | number",
        trailing: false,
        optional: true,
      });
    });

    test("handles parameter type overrides and sanitizes parameter names", () => {
      const route = parseRoute({
        method: "GET",
        path: "/items/:type?2fa",
        paramTypes: [{ name: "type", tsType: '"a" | "b"' }],
      });

      expect(route.params[0]!.name).toBe("type");
      expect(route.params[0]!.tsName).toBe("type_"); // reserved JS keyword
      expect(route.params[0]!.tsType).toBe('"a" | "b"');

      expect(route.query[0]!.name).toBe("param0_2fa");
      expect(route.query[0]!.tsName).toBe("param0_2fa");
    });

    test("throws an error for invalid HTTP methods", () => {
      expect(() =>
        parseRoute({
          method: "INVALID",
          path: "/test",
        }),
      ).toThrow("invalid HTTP method `INVALID`");
    });
  });

  describe("parseRoutes", () => {
    test("parses multiple routes and deduplicates generated handler names", () => {
      const defs = parseRoutes([
        { method: "GET", path: "/a/:id" },
        { method: "GET", path: "/a/:id" },
      ]);

      expect(defs.length).toBe(2);
      expect(defs[0]!.name).toBe("getAById");
      expect(defs[1]!.name).toBe("getAById_2");
    });
  });

  describe("generateTypes", () => {
    test("generates TypeScript interface definitions for routes", () => {
      const def = parseRoute({
        method: "GET",
        path: "/users/:id?<active>",
        response: "User",
      });

      const types = generateTypes([def], { typesName: "MyRouteTypes" });

      expect(types).toContain("export interface MyRouteTypes {");
      expect(types).toContain('"GET /users/:id": {');
      expect(types).toContain("id: string | number;");
      expect(types).toContain("active?: string | number;");
      expect(types).toContain("response: User;");
      expect(types).toContain(
        'export type ParamsOf<K extends RouteKey> = MyRouteTypes[K]["params"];',
      );
      expect(types).toContain("export interface RequestLike {");
      expect(types).not.toContain("export class");
    });

    test("respects serverImport option in generated types", () => {
      const def = parseRoute({ method: "GET", path: "/status" });
      const types = generateTypes([def], { serverImport: "@alloy-ts/server" });

      expect(types).toContain(
        'import type { RequestInput as RequestLike } from "@alloy-ts/server";',
      );
      expect(types).not.toContain("export interface RequestLike {");
    });
  });

  describe("generateClient", () => {
    test("generates a typed fetch client and uri helper", () => {
      const def = parseRoute({
        method: "GET",
        path: "/users/:id?<active>",
        response: "User",
      });

      const code = generateClient([def], {
        clientName: "UserClient",
        baseUrl: "https://api.example.com",
      });

      expect(code).toContain("export class UserClient {");
      expect(code).toContain('return this.options.baseUrl ?? "https://api.example.com";');
      expect(code).toContain("export function uri(");
      expect(code).toContain("async getUsersById(");
      expect(code).toContain('as ResponseOf<"GET /users/:id">;');
    });

    test("imports external types when typesImport option is specified", () => {
      const def = parseRoute({ method: "GET", path: "/users/:id" });
      const code = generateClient([def], { typesImport: "./types" });

      expect(code).toContain(
        'import type { ParamsOf, QueryOf, BodyOf, ResponseOf } from "./types";',
      );
      expect(code).not.toContain("export interface RouteTypes {");
    });
  });

  describe("generateModule", () => {
    test("generates a full module containing ROUTES, types, uri helper, and client", () => {
      const def = parseRoute({
        method: "POST",
        path: "/users",
        body: "CreateUser",
        response: "User",
      });

      const moduleCode = generateModule([def]);

      expect(moduleCode).toContain("export const ROUTES = {");
      expect(moduleCode).toContain('"POST /users": "/users",');
      expect(moduleCode).toContain("export interface RouteTypes {");
      expect(moduleCode).toContain("export function uri(");
      expect(moduleCode).toContain("export class RocketClient {");
      expect(moduleCode).toContain("async postUsers(");
    });

    test("allows disabling the banner comment", () => {
      const def = parseRoute({ method: "GET", path: "/" });
      const moduleCode = generateModule([def], { banner: false });

      expect(moduleCode).not.toContain("// Code generated by alloy-server-native");
    });
  });
});

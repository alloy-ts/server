import { describe, expect, test } from "vite-plus/test";
import {
  generateClient,
  generateModule,
  generateTypes,
  generateTypescriptClient,
  generateTypescriptInterface,
  httpMethods,
  parseRoute,
  parseRocketRoutePattern,
  parseRoutes,
  toRocketPath,
  translatePatternToAlloy,
} from "./index.ts";

describe("Rocket Codegen NAPI Bindings", () => {
  test("httpMethods returns supported HTTP methods", () => {
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
    expect(methods.length).toBe(9);
  });

  test("toRocketPath and translatePatternToAlloy translate paths", () => {
    expect(toRocketPath("/users/:id/posts/*")).toBe("/users/<id>/posts/<_..>");
    expect(toRocketPath("/users/<id..>")).toBe("/users/<id..>");
    expect(toRocketPath("/")).toBe("/");
    expect(toRocketPath("/api/:version/items")).toBe("/api/<version>/items");
    expect(translatePatternToAlloy("/users/:id")).toBe("/users/<id>");
  });

  describe("parseRoute and parseRocketRoutePattern", () => {
    test("parses simple GET route with path parameter", () => {
      const def = parseRoute({
        method: "get",
        path: "/users/:id",
      });

      expect(def.key).toBe("GET /users/:id");
      expect(def.method).toBe("GET");
      expect(def.path).toBe("/users/:id");
      expect(def.rocketPath).toBe("/users/<id>");
      expect(def.name).toBe("getUsersById");
      expect(def.params).toEqual([
        {
          name: "id",
          tsName: "id",
          kind: "Path",
          tsType: "string | number",
          trailing: false,
          optional: false,
        },
      ]);
      expect(def.query).toEqual([]);
      expect(def.body).toBeUndefined();
      expect(def.response).toBeUndefined();

      const defAlias = parseRocketRoutePattern({
        method: "get",
        path: "/users/:id",
      });
      expect(defAlias.key).toBe("GET /users/:id");
    });

    test("parses route with multi-segment trailing param and query parameters", () => {
      const def = parseRoute({
        method: "POST",
        path: "/files/*?<download>&<version>",
      });

      expect(def.key).toBe("POST /files/*");
      expect(def.method).toBe("POST");
      expect(def.path).toBe("/files/*");
      expect(def.rocketPath).toBe("/files/<_..>");
      expect(def.params).toHaveLength(1);
      expect(def.params[0]).toEqual({
        name: "_",
        tsName: "_",
        kind: "Path",
        tsType: "string[]",
        trailing: true,
        optional: true,
      });

      expect(def.query).toHaveLength(2);
      expect(def.query[0]).toEqual({
        name: "download",
        tsName: "download",
        kind: "Query",
        tsType: "string | number",
        trailing: false,
        optional: true,
      });
      expect(def.query[1]).toEqual({
        name: "version",
        tsName: "version",
        kind: "Query",
        tsType: "string | number",
        trailing: false,
        optional: true,
      });
    });

    test("applies parameter type overrides", () => {
      const def = parseRoute({
        method: "GET",
        path: "/users/:id?<active>",
        paramTypes: [{ name: "id", tsType: "number" }],
        queryTypes: [{ name: "active", tsType: "boolean" }],
      });

      expect(def.params[0]!.tsType).toBe("number");
      expect(def.query[0]!.tsType).toBe("boolean");
    });

    test("sanitizes reserved TypeScript keywords in parameter names", () => {
      const def = parseRoute({
        method: "GET",
        path: "/items/:type",
      });

      expect(def.params[0]!.name).toBe("type");
      expect(def.params[0]!.tsName).toBe("type_");
    });

    test("sanitizes parameter names starting with numbers", () => {
      const def = parseRoute({
        method: "GET",
        path: "/search?2fa",
      });

      expect(def.query[0]!.name).toBe("param0_2fa");
      expect(def.query[0]!.tsName).toBe("param0_2fa");
    });

    test("supports explicit handler name, body, and response types", () => {
      const def = parseRoute({
        method: "POST",
        path: "/accounts",
        name: "createAccount",
        body: "CreateAccountDto",
        response: "AccountDto",
      });

      expect(def.name).toBe("createAccount");
      expect(def.body).toBe("CreateAccountDto");
      expect(def.response).toBe("AccountDto");
    });

    test("throws error for invalid HTTP method", () => {
      expect(() =>
        parseRoute({
          method: "INVALID",
          path: "/",
        }),
      ).toThrow(/invalid HTTP method/);
    });
  });

  describe("parseRoutes", () => {
    test("parses route table and deduplicates handler names", () => {
      const defs = parseRoutes([
        { method: "GET", path: "/users/:id" },
        { method: "GET", path: "/users/:id" },
        { method: "POST", path: "/users", name: "createUser" },
      ]);

      expect(defs).toHaveLength(3);
      expect(defs[0]!.name).toBe("getUsersById");
      expect(defs[1]!.name).toBe("getUsersById_2");
      expect(defs[2]!.name).toBe("createUser");
    });
  });

  describe("generateTypes and generateTypescriptInterface", () => {
    test("generates TypeScript interfaces and type aliases", () => {
      const def = parseRoute({
        method: "GET",
        path: "/users/:id?<active>",
        response: "UserDto",
      });

      const types = generateTypes([def]);

      expect(types).toContain(
        "// Code generated by alloy-server-native (Rocket codegen). DO NOT EDIT.",
      );
      expect(types).toContain("export interface RequestLike {");
      expect(types).toContain("export interface RouteTypes {");
      expect(types).toContain('"GET /users/:id": {');
      expect(types).toContain("id: string | number;");
      expect(types).toContain("active?: string | number;");
      expect(types).toContain("response: UserDto;");
      expect(types).toContain(
        'export type ParamsOf<K extends RouteKey> = RouteTypes[K]["params"];',
      );
      expect(types).toContain("export interface RouteContext<K extends RouteKey>");
      expect(types).toContain("export type RouteHandler<K extends RouteKey>");

      const typesAlias = generateTypescriptInterface([def]);
      expect(typesAlias).toBe(types);
    });

    test("respects codegen options for types generation", () => {
      const def = parseRoute({ method: "GET", path: "/health" });

      const types = generateTypes([def], {
        typesName: "AppRouteTypes",
        serverImport: "@alloy-ts/server",
        banner: false,
      });

      expect(types).not.toContain("DO NOT EDIT");
      expect(types).toContain(
        'import type { RequestInput as RequestLike } from "@alloy-ts/server";',
      );
      expect(types).toContain("export interface AppRouteTypes {");
    });
  });

  describe("generateClient and generateTypescriptClient", () => {
    test("generates typed fetch client class and uri helper", () => {
      const def = parseRoute({
        method: "GET",
        path: "/users/:id",
        response: "UserDto",
      });

      const client = generateClient([def]);

      expect(client).toContain("export class RocketClient {");
      expect(client).toContain("export function uri(");
      expect(client).toContain("async getUsersById(");
      expect(client).toContain('as ResponseOf<"GET /users/:id">;');

      const clientAlias = generateTypescriptClient([def]);
      expect(clientAlias).toBe(client);
    });

    test("respects codegen options for client generation", () => {
      const def = parseRoute({ method: "GET", path: "/status" });

      const client = generateClient([def], {
        clientName: "ApiClient",
        baseUrl: "https://api.example.com",
        typesImport: "./api.types",
      });

      expect(client).toContain(
        'import type { ParamsOf, QueryOf, BodyOf, ResponseOf } from "./api.types";',
      );
      expect(client).toContain("export class ApiClient {");
      expect(client).toContain("https://api.example.com");
    });
  });

  describe("generateModule", () => {
    test("generates complete module with ROUTES, types, uri, and client", () => {
      const def = parseRoute({
        method: "GET",
        path: "/users/:id",
        response: "UserDto",
      });

      const mod = generateModule([def]);

      expect(mod).toContain("export const ROUTES = {");
      expect(mod).toContain('"GET /users/:id": "/users/:id",');
      expect(mod).toContain("export interface RouteTypes {");
      expect(mod).toContain("export function uri(");
      expect(mod).toContain("export class RocketClient {");
    });
  });
});

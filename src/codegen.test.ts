import { describe, expect, it } from "vite-plus/test";
import {
  generateClient,
  generateModule,
  generateTypes,
  httpMethods,
  type ParamKind,
  parseRoute,
  parseRoutes,
  toRocketPath,
} from "./main.ts";

describe("codegen NAPI bindings", () => {
  it("httpMethods returns all accepted HTTP methods", () => {
    const methods = httpMethods();
    expect(methods).toContain("GET");
    expect(methods).toContain("POST");
    expect(methods).toContain("PUT");
    expect(methods).toContain("DELETE");
    expect(methods).toContain("PATCH");
    expect(methods).toContain("OPTIONS");
    expect(methods).toContain("HEAD");
    expect(methods).toContain("TRACE");
    expect(methods).toContain("CONNECT");
    expect(methods.length).toBe(9);
  });

  it("parseRoute parses Bun/Alloy style paths", () => {
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

    expect(def.params[0]).toMatchObject({
      name: "id",
      tsName: "id",
      kind: "Path" as ParamKind,
      trailing: false,
      optional: false,
    });

    expect(def.params[1]).toMatchObject({
      name: "_",
      tsName: "_",
      kind: "Path" as ParamKind,
      trailing: true,
      optional: true,
    });
  });

  it("parseRoute parses Rocket style path and query parameters", () => {
    const def = parseRoute({
      method: "POST",
      path: "/users/<id..>?<q>&closed",
    });

    expect(def.path).toBe("/users/:id");
    expect(def.rocketPath).toBe("/users/<id..>");
    expect(def.key).toBe("POST /users/:id");
    expect(def.params[0]?.trailing).toBe(true);

    expect(def.query.length).toBe(2);
    expect(def.query[0]?.name).toBe("q");
    expect(def.query[1]?.name).toBe("closed");
    expect(def.query.every((q) => q.optional)).toBe(true);
  });

  it("parseRoute throws error on invalid HTTP method", () => {
    expect(() =>
      parseRoute({
        method: "FETCH",
        path: "/",
      }),
    ).toThrow(/invalid HTTP method/);
  });

  it("parseRoute handles param and query type overrides and reserved words", () => {
    const def = parseRoute({
      method: "GET",
      path: "/items/:type?2fa",
      paramTypes: [
        {
          name: "type",
          tsType: '"a" | "b"',
        },
      ],
      queryTypes: [
        {
          name: "param0_2fa",
          tsType: "boolean",
        },
      ],
    });

    // "type" is reserved in TypeScript, so tsName becomes "type_"
    expect(def.params[0]?.tsName).toBe("type_");
    expect(def.params[0]?.tsType).toBe('"a" | "b"');
    expect(def.query[0]?.tsName).toBe("param0_2fa");
    expect(def.query[0]?.tsType).toBe("boolean");
  });

  it("parseRoutes resolves multiple routes and dedupes handler names", () => {
    const defs = parseRoutes([
      { method: "GET", path: "/a/:id" },
      { method: "GET", path: "/a/:id" },
      { method: "POST", path: "/a/:id", name: "getAById" },
    ]);

    expect(defs.length).toBe(3);
    expect(defs[0]?.name).toBe("getAById");
    expect(defs[1]?.name).toBe("getAById_2");
    expect(defs[2]?.name).toBe("getAById_3");
  });

  it("toRocketPath translates Alloy/Bun patterns to Rocket syntax", () => {
    expect(toRocketPath("/a/:b/*")).toBe("/a/<b>/<_..>");
    expect(toRocketPath("/a/<b..>")).toBe("/a/<b..>");
    expect(toRocketPath("/users/:id?query")).toBe("/users/<id>");
  });

  it("generateTypes emits TypeScript type interfaces", () => {
    const def = parseRoute({
      method: "GET",
      path: "/users/:id?<active>",
      response: "User",
    });

    const code = generateTypes([def]);
    expect(code).toContain("export interface RouteTypes {");
    expect(code).toContain('"GET /users/:id": {');
    expect(code).toContain("id: string | number;");
    expect(code).toContain("active?: string | number;");
    expect(code).toContain("response: User;");
    expect(code).toContain("export type ParamsOf<K extends RouteKey>");
    expect(code).toContain("export interface RouteContext");
  });

  it("generateTypes respects serverImport option", () => {
    const def = parseRoute({ method: "GET", path: "/test" });
    const code = generateTypes([def], { serverImport: "@alloy-ts/server" });
    expect(code).toContain('import type { RequestInput as RequestLike } from "@alloy-ts/server";');
  });

  it("generateClient emits typed client class", () => {
    const def = parseRoute({
      method: "GET",
      path: "/users/:id",
      response: "User",
    });

    const code = generateClient([def], { clientName: "UserClient" });
    expect(code).toContain("export class UserClient {");
    expect(code).toContain("export function uri(");
    expect(code).toContain("async getUsersById(");
    expect(code).toContain('as ResponseOf<"GET /users/:id">;');
  });

  it("generateModule emits full module with ROUTES table, types, uri, and client", () => {
    const defs = parseRoutes([
      { method: "GET", path: "/users/:id", response: "User" },
      { method: "POST", path: "/users", body: "CreateUser", response: "User" },
    ]);

    const code = generateModule(defs, {
      baseUrl: "http://localhost:3000",
      clientName: "ApiClient",
    });

    expect(code).toContain("export const ROUTES = {");
    expect(code).toContain('"GET /users/:id": "/users/:id",');
    expect(code).toContain("export interface RouteTypes {");
    expect(code).toContain("export function uri(");
    expect(code).toContain("export class ApiClient {");
    expect(code).toContain('http://localhost:3000"');
    expect(code).toContain("async getUsersById(");
    expect(code).toContain("async postUsers(");
  });
});

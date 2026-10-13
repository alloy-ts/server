import { expect, test } from "vite-plus/test";
import {
  buildRouteUri,
  del,
  generateRoutesTypeDefinitions,
  generateTsRouteTypes,
  get,
  getRegisteredRoutesMeta,
  parseRoutePattern,
  post,
  put,
  route,
  uri,
} from "./codegen.ts";

test("parseRoutePattern extracts path and query parameters", () => {
  const info = parseRoutePattern("/api/v1/users/<id>/<action..>?<ref>&<verbose..>");
  expect(info.uri).toBe("/api/v1/users/<id>/<action..>?<ref>&<verbose..>");
  expect(info.pathParams).toEqual([
    { name: "id", isTrailing: false },
    { name: "action", isTrailing: true },
  ]);
  expect(info.queryParams).toEqual(["ref", "verbose"]);
});

test("parseRoutePattern handles Express/Bun colon syntax", () => {
  const info = parseRoutePattern("/posts/:id/:slug");
  expect(info.pathParams).toEqual([
    { name: "id", isTrailing: false },
    { name: "slug", isTrailing: false },
  ]);
});

test("buildRouteUri and uri construct encoded URLs", () => {
  const url1 = uri("/users/:id/profile", { id: "42" });
  expect(url1).toBe("/users/42/profile");

  const url2 = buildRouteUri("/search?<q>&<page>", { q: "hello world", page: "2" });
  expect(url2).toBe("/search?q=hello%20world&page=2");
});

test("generateTsRouteTypes emits valid TypeScript interface definitions", () => {
  const routeInfo = parseRoutePattern("/items/<id>?<filter>");
  routeInfo.method = "GET";

  const code = generateTsRouteTypes([routeInfo]);
  expect(code).toContain("export interface ApiRoutes");
  expect(code).toContain('"GET /items/<id>?<filter>": {');
  expect(code).toContain("id: string;");
  expect(code).toContain("filter?: string;");

  const codeFromPatterns = generateRoutesTypeDefinitions(["/users/<id>"]);
  expect(codeFromPatterns).toContain("export interface ApiRoutes");
  expect(codeFromPatterns).toContain("id: string;");
});

test("route registration helpers and getRegisteredRoutesMeta", () => {
  get("/test/get", () => "ok");
  post("/test/post", () => "ok");
  put("/test/put", () => "ok");
  del("/test/del", () => "ok");
  route("PATCH", "/test/patch", () => "ok");

  const meta = getRegisteredRoutesMeta();
  expect(meta.some((m) => m.path === "/test/get" && m.method === "GET")).toBe(true);
  expect(meta.some((m) => m.path === "/test/post" && m.method === "POST")).toBe(true);
  expect(meta.some((m) => m.path === "/test/put" && m.method === "PUT")).toBe(true);
  expect(meta.some((m) => m.path === "/test/del" && m.method === "DELETE")).toBe(true);
  expect(meta.some((m) => m.path === "/test/patch" && m.method === "PATCH")).toBe(true);
});

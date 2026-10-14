import { expect, test } from "vite-plus/test";
import { buildRouteUri, generateTsRouteTypes, parseRoutePattern, uri } from "../dist/index.js";

test("parseRoutePattern parses single and trailing path parameters and query parameters", () => {
  const routeInfo = parseRoutePattern("/users/<id>/<action..>?<ref>&<sort>");
  expect(routeInfo.method).toBe("GET");
  expect(routeInfo.uri).toBe("/users/<id>/<action..>?<ref>&<sort>");
  expect(routeInfo.pathParams).toEqual([
    { name: "id", isTrailing: false },
    { name: "action", isTrailing: true },
  ]);
  expect(routeInfo.queryParams).toEqual(["ref", "sort"]);
});

test("parseRoutePattern parses colon-style parameters", () => {
  const routeInfo = parseRoutePattern("/posts/:category/:id?:page");
  expect(routeInfo.pathParams).toEqual([
    { name: "category", isTrailing: false },
    { name: "id", isTrailing: false },
  ]);
  expect(routeInfo.queryParams).toEqual(["page"]);
});

test("uri replaces dynamic path parameters and URL encodes values", () => {
  const userUri = uri("/users/:id", { id: "42" });
  expect(userUri).toBe("/users/42");

  const encodedUri = uri("/users/<name>", { name: "John Doe" });
  expect(encodedUri).toBe("/users/John%20Doe");
});

test("buildRouteUri constructs complete URL with dynamic path and query parameters", () => {
  const result = buildRouteUri("/search/<category>/<item..>?<page>&<sort>", {
    category: "electronics",
    item: "laptops/pro",
    page: 2,
    sort: "asc",
  });
  expect(result).toBe("/search/electronics/laptops%2Fpro?page=2&sort=asc");
});

test("generateTsRouteTypes generates valid TypeScript interface definitions", () => {
  const patterns = [
    "/api/v1/health",
    "/api/v1/users/<id>?<ref>",
    "/api/v1/posts/<category>/<id..>?<page>",
  ];

  const tsDefs = generateTsRouteTypes(patterns);
  expect(tsDefs).toContain("export interface ApiRoutes");
  expect(tsDefs).toContain('"GET /api/v1/health":');
  expect(tsDefs).toContain("id: string;");
  expect(tsDefs).toContain("ref?: string;");
  expect(tsDefs).toContain("page?: string;");
});

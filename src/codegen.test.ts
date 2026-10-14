import { expect, test } from "vite-plus/test";
import {
  parseRoutePattern,
  buildRouteUri,
  parseCatcherCode,
  generateRouteManifest,
} from "../index.js";

test("codegen.rs - parseRoutePattern with Express parameters", () => {
  const result = parseRoutePattern("/users/:id/posts/:postId");
  expect(result.rawPattern).toBe("/users/:id/posts/:postId");
  expect(result.rocketPattern).toBe("/users/<id>/posts/<postId>");
  expect(result.pathParams).toEqual(["id", "postId"]);
  expect(result.isWildcard).toBe(false);
});

test("codegen.rs - parseRoutePattern with Rocket dynamic segments and wildcards", () => {
  const result = parseRoutePattern("/api/*");
  expect(result.rawPattern).toBe("/api/*");
  expect(result.rocketPattern).toBe("/api/<_..>");
  expect(result.isWildcard).toBe(true);
});

test("codegen.rs - buildRouteUri interpolates colon and angle parameters with URL encoding", () => {
  const uri1 = buildRouteUri("/users/:id", { id: "123" });
  expect(uri1).toBe("/users/123");

  const uri2 = buildRouteUri("/search/:query", { query: "rocket & napi web" });
  expect(uri2).toBe("/search/rocket%20%26%20napi%20web");

  const uri3 = buildRouteUri("/posts/<category>/<slug>", {
    category: "tech",
    slug: "rocket-v0.5",
  });
  expect(uri3).toBe("/posts/tech/rocket-v0.5");
});

test("codegen.rs - parseCatcherCode parses status codes and default keywords", () => {
  expect(parseCatcherCode("404")).toBe(404);
  expect(parseCatcherCode("500")).toBe(500);
  expect(parseCatcherCode("default")).toBeNull();
  expect(parseCatcherCode("_")).toBeNull();
  expect(parseCatcherCode("DEFAULT")).toBeNull();
  expect(parseCatcherCode("invalid")).toBeNull();
});

test("codegen.rs - generateRouteManifest compiles manifest array", () => {
  const manifest = generateRouteManifest(["/health", "/users/:id", "/api/*"]);

  expect(manifest.length).toBe(3);
  expect(manifest[0]).toEqual({
    rawPattern: "/health",
    rocketPattern: "/health",
    pathParams: [],
    isWildcard: false,
  });

  expect(manifest[1]?.pathParams).toEqual(["id"]);
  expect(manifest[2]?.isWildcard).toBe(true);
});

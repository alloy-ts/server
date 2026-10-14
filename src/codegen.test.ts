import { expect, test } from "vite-plus/test";
import { generateRoutesManifest, buildUri, createRouteMetadata } from "../dist/index.js";

test("createRouteMetadata constructs RouteMetadata object", () => {
  const meta = createRouteMetadata("GET", "/users/<id>", "get_user", 1, "json");
  expect(meta.method).toBe("GET");
  expect(meta.uri).toBe("/users/<id>");
  expect(meta.name).toBe("get_user");
  expect(meta.rank).toBe(1);
  expect(meta.format).toBe("json");
});

test("generateRoutesManifest outputs TypeScript route manifest code", () => {
  const routes = [
    {
      method: "GET",
      uri: "/api/status",
      name: "status",
      rank: 0,
      format: "text/plain",
    },
  ];
  const catchers = [{ code: 404, name: "not_found" }];

  const manifest = generateRoutesManifest(routes, catchers);
  expect(manifest).toContain("export const routesManifest = [");
  expect(manifest).toContain('"GET"');
  expect(manifest).toContain('"/api/status"');
  expect(manifest).toContain("export const catchersManifest = [");
  expect(manifest).toContain("404");
  expect(manifest).toContain('"not_found"');
});

test("buildUri replaces parameterized route placeholders", () => {
  const uri = buildUri("/users/<id>/posts/:postId", {
    id: "42",
    postId: "10",
  });
  expect(uri).toBe("/users/42/posts/10");
});

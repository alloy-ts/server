import { expect, test } from "vite-plus/test";
import {
  parseRouteAttribute,
  parseCatcherAttribute,
  buildUri,
  uri,
  generateTsRouteManifest,
} from "../dist/index.js";

test("parseRouteAttribute formats route metadata correctly", () => {
  const meta = parseRouteAttribute("get", "/users/:id", "get_user", 1, "json");
  expect(meta.method).toBe("GET");
  expect(meta.uri).toBe("/users/:id");
  expect(meta.name).toBe("get_user");
  expect(meta.rank).toBe(1);
  expect(meta.format).toBe("json");
});

test("parseCatcherAttribute parses status code or default catcher", () => {
  const notFound = parseCatcherAttribute("404", "not_found");
  expect(notFound.code).toBe(404);
  expect(notFound.name).toBe("not_found");

  const defaultCatcher = parseCatcherAttribute("default", "default_handler");
  expect(defaultCatcher.code).toBeUndefined();
  expect(defaultCatcher.name).toBe("default_handler");
});

test("buildUri and uri perform parameter interpolation and URL encoding", () => {
  const uri1 = buildUri("/users/<id>/posts/<post_id>", {
    id: "10",
    post_id: "hello world & rust",
  });
  expect(uri1).toBe("/users/10/posts/hello%20world%20%26%20rust");

  const uri2 = uri("/users/:id/posts/:post_id", {
    id: "42",
    post_id: "foo/bar",
  });
  expect(uri2).toBe("/users/42/posts/foo%2Fbar");
});

test("generateTsRouteManifest outputs TypeScript route mapping", () => {
  const routes = [
    parseRouteAttribute("get", "/api/status", "status_handler", null, null),
    parseRouteAttribute("post", "/api/users", "create_user", null, null),
  ];
  const manifest = generateTsRouteManifest(routes);
  expect(manifest).toContain("// Auto-generated Rocket Route Manifest");
  expect(manifest).toContain('"/api/status": { method: "GET", name: "status_handler" }');
  expect(manifest).toContain('"/api/users": { method: "POST", name: "create_user" }');
});

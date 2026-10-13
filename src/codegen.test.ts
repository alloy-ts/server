import { expect, test } from "vite-plus/test";
import {
  Alloy,
  generateTsRouteManifest,
  parseCatcherAttribute,
  parseRouteAttribute,
} from "./main.ts";

test("parseRouteAttribute parses route metadata correctly", () => {
  const metadata = parseRouteAttribute("get", "/api/items", "list_items", 1, "json");
  expect(metadata).toEqual({
    method: "GET",
    uri: "/api/items",
    name: "list_items",
    rank: 1,
    format: "json",
  });
});

test("parseCatcherAttribute parses status codes and default catchers", () => {
  const notFound = parseCatcherAttribute("404", "not_found");
  expect(notFound).toEqual({ code: 404, name: "not_found" });

  const defaultCatcher = parseCatcherAttribute("default", "default_handler");
  expect(defaultCatcher).toEqual({ code: undefined, name: "default_handler" });
});

test("generateTsRouteManifest generates a valid TypeScript manifest code string", () => {
  const manifest = generateTsRouteManifest([
    { method: "GET", uri: "/", name: "index" },
    { method: "POST", uri: "/items", name: "create_item" },
  ]);

  expect(manifest).toContain("// Auto-generated Rocket Route Manifest");
  expect(manifest).toContain('"/"');
  expect(manifest).toContain('"GET"');
  expect(manifest).toContain('"/items"');
  expect(manifest).toContain('"create_item"');
});

test("Alloy.uri works as codegen URI builder", () => {
  const built = Alloy.uri("/user/<id>/post/<post_id>", { id: "123", post_id: "a/b" });
  expect(built).toBe("/user/123/post/a%2Fb");
});

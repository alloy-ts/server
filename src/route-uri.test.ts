import { expect, test } from "vite-plus/test";
import { RouteUri } from "../dist/index.js";

test("RouteUri constructor initializes base, unmountedOrigin, and origin", () => {
  const uri = new RouteUri("/base", "/foo/<bar>");
  expect(uri.base).toBe("/base");
  expect(uri.unmountedOrigin).toBe("/foo/<bar>");
  expect(uri.origin).toBe("/base/foo/<bar>");
  expect(uri.asStr()).toBe("/base/foo/<bar>");
  expect(uri.path()).toBe("/base/foo/<bar>");
});

test("RouteUri handles default base '/' and query parameters", () => {
  const uri = new RouteUri(null, "/items?id=123");
  expect(uri.base).toBe("/");
  expect(uri.path()).toBe("/items");
  expect(uri.query()).toBe("id=123");
  expect(uri.asStr()).toBe("/items?id=123");
});

test("RouteUri calculates default rank for static and dynamic URIs", () => {
  const staticUri = new RouteUri("/", "/a/b/c");
  expect(staticUri.defaultRank()).toBe(-9);

  const queryUri = new RouteUri("/", "/a/b?q=1");
  expect(queryUri.defaultRank()).toBe(-12);
});

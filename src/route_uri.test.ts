import { expect, test } from "vite-plus/test";
import { createRouteUri } from "../dist/index.js";

test("createRouteUri parses base, unmountedOrigin, origin, path, and query", () => {
  const routeUri = createRouteUri("/foo", "/bar?a=1");
  expect(routeUri.base).toBe("/foo");
  expect(routeUri.unmountedOrigin).toBe("/bar?a=1");
  expect(routeUri.origin).toBe("/foo/bar?a=1");
  expect(routeUri.path).toBe("/foo/bar");
  expect(routeUri.query).toBe("a=1");
});

test("createRouteUri handles root base and missing query", () => {
  const routeUri = createRouteUri("/", "/items");
  expect(routeUri.base).toBe("/");
  expect(routeUri.unmountedOrigin).toBe("/items");
  expect(routeUri.origin).toBe("/items");
  expect(routeUri.path).toBe("/items");
  expect(routeUri.query).toBeUndefined();
});

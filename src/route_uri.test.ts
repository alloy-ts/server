import { expect, test } from "vite-plus/test";
import { RouteUri } from "../dist/index.js";

test("RouteUri constructor and getters", () => {
  const routeUri = new RouteUri("/", "/foo/bar?a=1");
  expect(routeUri.base).toBe("/");
  expect(routeUri.path).toBe("/foo/bar");
  expect(routeUri.query).toBe("a=1");
  expect(routeUri.asStr()).toBe("/foo/bar?a=1");
  expect(routeUri.isNormalized()).toBe(true);
});

test("RouteUri mapBase updates base and canonical origin path", () => {
  const routeUri = new RouteUri("/", "/foo/bar?a=1");
  expect(routeUri.base).toBe("/");

  routeUri.mapBase("/boo");
  expect(routeUri.base).toBe("/boo");
  expect(routeUri.path).toBe("/boo/foo/bar");
  expect(routeUri.asStr()).toBe("/boo/foo/bar?a=1");
});

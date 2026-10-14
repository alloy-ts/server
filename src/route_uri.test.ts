import { expect, test } from "vite-plus/test";
import { RouteUri } from "../dist/index.js";

test("RouteUri base, unmountedOrigin, and origin fields", () => {
  const uri1 = new RouteUri("/", "/foo/<bar>");
  expect(uri1.base).toBe("/");
  expect(uri1.unmountedOrigin).toBe("/foo/<bar>");
  expect(uri1.origin).toBe("/foo/<bar>");
  expect(uri1.asStr()).toBe("/foo/<bar>");

  const uri2 = new RouteUri("/base", "/foo/<bar>");
  expect(uri2.base).toBe("/base");
  expect(uri2.unmountedOrigin).toBe("/foo/<bar>");
  expect(uri2.origin).toBe("/base/foo/<bar>");
  expect(uri2.asStr()).toBe("/base/foo/<bar>");
});

test("RouteUri path and query parsing", () => {
  const uri = new RouteUri("/", "/foo/bar?a=1&b=2");
  expect(uri.path()).toBe("/foo/bar");
  expect(uri.query()).toBe("a=1&b=2");

  const noQuery = new RouteUri("/", "/foo/bar");
  expect(noQuery.path()).toBe("/foo/bar");
  expect(noQuery.query()).toBeNull();
});

test("RouteUri defaultRank calculation", () => {
  const staticRoute = new RouteUri("/", "/api/status?v=1");
  expect(staticRoute.defaultRank()).toBe(-12);

  const wildRoute = new RouteUri("/", "/<b>/<c>");
  expect(wildRoute.defaultRank()).toBe(-1);
});

import { expect, test } from "vite-plus/test";
import { RouteUri } from "../dist/index.js";

test("RouteUri initializes base, unmountedOrigin, and origin", () => {
  const uri = new RouteUri("/", "/foo/<bar>");
  expect(uri.base).toBe("/");
  expect(uri.unmountedOrigin).toBe("/foo/<bar>");
  expect(uri.origin).toBe("/foo/<bar>");
  expect(uri.asStr()).toBe("/foo/<bar>");
  expect(uri.path()).toBe("/foo/<bar>");
  expect(uri.query()).toBeFalsy();
});

test("RouteUri with custom mount point concatenates base and unmounted origin", () => {
  const uri = RouteUri.create("/base", "/foo/bar?a=1");
  expect(uri.base).toBe("/base");
  expect(uri.unmountedOrigin).toBe("/foo/bar?a=1");
  expect(uri.origin).toBe("/base/foo/bar?a=1");
  expect(uri.path()).toBe("/base/foo/bar");
  expect(uri.query()).toBe("a=1");
});

test("RouteUri calculates default rank", () => {
  const staticRoute = new RouteUri("/", "/foo/bar");
  expect(staticRoute.defaultRank()).toBe(-9);

  const queryRoute = new RouteUri("/", "/foo/bar?a=1");
  expect(queryRoute.defaultRank()).toBe(-12);

  const dynamicRoute = new RouteUri("/", "/users/<id>");
  expect(dynamicRoute.defaultRank()).toBe(-5);
});

test("RouteUri checks normalization", () => {
  const uri = new RouteUri("/", "/a/b/c");
  expect(uri.isNormalized()).toBe(true);
});

import { expect, test } from "vite-plus/test";
import { RouteUri } from "../dist/index.js";

test("RouteUri parses base, unmounted_origin, origin, path, and query", () => {
  const uri = new RouteUri("/", "/foo/bar?a=1");
  expect(uri.base).toBe("/");
  expect(uri.unmountedOrigin).toBe("/foo/bar?a=1");
  expect(uri.origin).toBe("/foo/bar?a=1");
  expect(uri.path).toBe("/foo/bar");
  expect(uri.query).toBe("a=1");
  expect(uri.asStr()).toBe("/foo/bar?a=1");
});

test("RouteUri supports custom base mount points", () => {
  const uri = new RouteUri("/boo", "/foo/bar?a=1");
  expect(uri.base).toBe("/boo");
  expect(uri.unmountedOrigin).toBe("/foo/bar?a=1");
  expect(uri.origin).toBe("/boo/foo/bar?a=1");
  expect(uri.path).toBe("/boo/foo/bar");
  expect(uri.query).toBe("a=1");
});

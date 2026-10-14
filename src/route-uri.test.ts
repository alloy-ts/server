import { expect, test } from "vite-plus/test";
import { RouteUri } from "../dist/index.js";

test("RouteUri parses base, unmounted origin, origin, path, and query", () => {
  const uri = new RouteUri("/", "/foo/bar?a=1");
  expect(uri.base).toBe("/");
  expect(uri.unmountedOrigin).toBe("/foo/bar?a=1");
  expect(uri.origin).toBe("/foo/bar?a=1");
  expect(uri.path).toBe("/foo/bar");
  expect(uri.query).toBe("a=1");
  expect(uri.asStr()).toBe("/foo/bar?a=1");

  const mountedUri = new RouteUri("/boo", "/foo/bar?a=1");
  expect(mountedUri.base).toBe("/boo");
  expect(mountedUri.unmountedOrigin).toBe("/foo/bar?a=1");
  expect(mountedUri.origin).toBe("/boo/foo/bar?a=1");
  expect(mountedUri.path).toBe("/boo/foo/bar");
  expect(mountedUri.query).toBe("a=1");
});

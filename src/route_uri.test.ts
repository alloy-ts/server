import { expect, test } from "vite-plus/test";
import { RouteUri } from "../dist/index.js";

test("RouteUri base, unmounted_origin, and canonical origin construction", () => {
  const uri1 = new RouteUri("/", "/foo/bar?a=1");
  expect(uri1.base).toBe("/");
  expect(uri1.unmountedOrigin).toBe("/foo/bar?a=1");
  expect(uri1.origin).toBe("/foo/bar?a=1");
  expect(uri1.path()).toBe("/foo/bar");
  expect(uri1.query()).toBe("a=1");
  expect(uri1.asStr()).toBe("/foo/bar?a=1");

  const uri2 = new RouteUri("/base", "/foo/<bar>");
  expect(uri2.base).toBe("/base");
  expect(uri2.unmountedOrigin).toBe("/foo/<bar>");
  expect(uri2.origin).toBe("/base/foo/<bar>");
  expect(uri2.path()).toBe("/base/foo/<bar>");
  expect(uri2.query()).toBeNull();
  expect(uri2.asStr()).toBe("/base/foo/<bar>");
});

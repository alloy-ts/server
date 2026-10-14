import { expect, test } from "vite-plus/test";
import { RouteUri } from "../dist/index.js";

test("RouteUri NAPI class parses base, path, query, and asStr", () => {
  const uri = new RouteUri("/", "/foo/bar?a=1");
  expect(uri.base()).toBe("/");
  expect(uri.path()).toBe("/foo/bar");
  expect(uri.query()).toBe("a=1");
  expect(uri.asStr()).toBe("/foo/bar?a=1");

  const uriWithBase = new RouteUri("/base", "/foo/<bar>");
  expect(uriWithBase.base()).toBe("/base");
  expect(uriWithBase.path()).toBe("/base/foo/<bar>");
  expect(uriWithBase.unmountedOrigin).toBe("/foo/<bar>");
  expect(uriWithBase.origin).toBe("/base/foo/<bar>");
});

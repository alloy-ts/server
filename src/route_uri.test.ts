import { expect, test } from "vite-plus/test";
import { RouteUri } from "../dist/index.js";

test("RouteUri parses base, unmounted origin, origin, path, and query", () => {
  const uri1 = new RouteUri(null, "/foo/bar?a=1");
  expect(uri1.base).toBe("/");
  expect(uri1.unmountedOrigin).toBe("/foo/bar?a=1");
  expect(uri1.origin).toBe("/foo/bar?a=1");
  expect(uri1.path).toBe("/foo/bar");
  expect(uri1.query).toBe("a=1");
  expect(uri1.asStr).toBe("/foo/bar?a=1");

  const uri2 = new RouteUri("/boo", "/foo/bar?a=1");
  expect(uri2.base).toBe("/boo");
  expect(uri2.unmountedOrigin).toBe("/foo/bar?a=1");
  expect(uri2.origin).toBe("/boo/foo/bar?a=1");
  expect(uri2.path).toBe("/boo/foo/bar");
  expect(uri2.query).toBe("a=1");
  expect(uri2.asStr).toBe("/boo/foo/bar?a=1");
});

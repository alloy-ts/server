import { expect, test } from "vite-plus/test";
import { RouteUri } from "../dist/index.js";

test("RouteUri parses default base and origin without query", () => {
  const uri = new RouteUri("/", "/foo/<bar>");

  expect(uri.base).toBe("/");
  expect(uri.unmountedOrigin).toBe("/foo/<bar>");
  expect(uri.origin).toBe("/foo/<bar>");
  expect(uri.path).toBe("/foo/<bar>");
  expect(uri.query).toBeNull();
  expect(uri.asStr()).toBe("/foo/<bar>");
});

test("RouteUri combines base mount point and origin with query", () => {
  const uri = new RouteUri("/base", "/foo/<bar>?a=1");

  expect(uri.base).toBe("/base");
  expect(uri.unmountedOrigin).toBe("/foo/<bar>?a=1");
  expect(uri.origin).toBe("/base/foo/<bar>?a=1");
  expect(uri.path).toBe("/base/foo/<bar>");
  expect(uri.query).toBe("a=1");
  expect(uri.asStr()).toBe("/base/foo/<bar>?a=1");
});

test("RouteUri throws on base mount point with query", () => {
  expect(() => new RouteUri("/base?query=1", "/foo")).toThrow();
});

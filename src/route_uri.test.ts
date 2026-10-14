import { expect, test } from "vite-plus/test";
import { RouteUri } from "../dist/index.js";

test("RouteUri base, unmountedOrigin, and origin properties", () => {
  const uri = new RouteUri("/", "/foo/<bar>");
  expect(uri.base).toBe("/");
  expect(uri.unmountedOrigin).toBe("/foo/<bar>");
  expect(uri.origin).toBe("/foo/<bar>");
  expect(uri.path()).toBe("/foo/<bar>");
  expect(uri.query()).toBeNull();
  expect(uri.asStr()).toBe("/foo/<bar>");
  expect(uri.defaultRank).toBe(-5);
});

test("RouteUri with custom mount point base", () => {
  const mountedUri = new RouteUri("/base", "/foo/<bar>?a=1");
  expect(mountedUri.base).toBe("/base");
  expect(mountedUri.unmountedOrigin).toBe("/foo/<bar>?a=1");
  expect(mountedUri.origin).toBe("/base/foo/<bar>?a=1");
  expect(mountedUri.path()).toBe("/base/foo/<bar>");
  expect(mountedUri.query()).toBe("a=1");
  expect(mountedUri.asStr()).toBe("/base/foo/<bar>?a=1");
});

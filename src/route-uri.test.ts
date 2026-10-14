import { expect, test } from "vite-plus/test";
import { RouteUri } from "../dist/index.js";

test("RouteUri NAPI class parses mount points and route URIs", () => {
  const uri = new RouteUri("/base", "/foo/bar?a=1");
  expect(uri.base).toBe("/base");
  expect(uri.path).toBe("/base/foo/bar");
  expect(uri.query).toBe("a=1");
  expect(uri.asStr()).toBe("/base/foo/bar?a=1");

  const staticUri = new RouteUri("/", "/?foo");
  expect(staticUri.defaultRank).toBe(-12);

  const defaultUri = new RouteUri("/", "/");
  expect(defaultUri.defaultRank).toBe(-9);
});

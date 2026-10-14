import { expect, test } from "vite-plus/test";
import { RouteUri } from "../index.js";

test("route-uri.ts - RouteUri constructor and properties", () => {
  const uri1 = new RouteUri("/", "/foo/bar?a=1");
  expect(uri1.base).toBe("/");
  expect(uri1.path).toBe("/foo/bar");
  expect(uri1.query).toBe("a=1");
  expect(uri1.asStr).toBe("/foo/bar?a=1");
  expect(typeof uri1.defaultRank).toBe("number");

  const uri2 = new RouteUri("/base", "/users/:id");
  expect(uri2.base).toBe("/base");
  expect(uri2.path).toBe("/base/users/:id");
});

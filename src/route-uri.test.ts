import { expect, test } from "vite-plus/test";
import { RouteUri } from "../dist/index.js";

test("RouteUri initializes base, unmountedOrigin, and canonical origin", () => {
  const uri = new RouteUri("/base", "/foo/<bar>");
  expect(uri.base).toBe("/base");
  expect(uri.unmountedOrigin).toBe("/foo/<bar>");
  expect(uri.origin).toBe("/base/foo/<bar>");
  expect(uri.path).toBe("/base/foo/<bar>");
  expect(uri.asStr()).toBe("/base/foo/<bar>");
});

test("RouteUri extracts query parameter string", () => {
  const uri = new RouteUri("/", "/search?q=rocket");
  expect(uri.query).toBe("q=rocket");
  expect(uri.path).toBe("/search");
});

import { expect, test } from "vite-plus/test";
import { RouteUri } from "../dist/index.js";

test("RouteUri constructor and methods", () => {
  const routeUri = new RouteUri("/base", "/foo/<bar>?a=1");
  expect(routeUri.base).toBe("/base");
  expect(routeUri.path()).toBe("/base/foo/<bar>?a=1");
  expect(routeUri.query()).toBe("a=1");
  expect(routeUri.asStr()).toBe("/base/foo/<bar>?a=1");
});

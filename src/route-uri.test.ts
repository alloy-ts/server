import { expect, test } from "vite-plus/test";
import { type RouteUriInfo, createRouteUri } from "../dist/index.js";

test("RouteUri creation and origin calculation", () => {
  const routeUri1: RouteUriInfo = createRouteUri(undefined, "/foo/<bar>");
  expect(routeUri1.base).toBe("/");
  expect(routeUri1.unmountedOrigin).toBe("/foo/<bar>");
  expect(routeUri1.origin).toBe("/foo/<bar>");
  expect(routeUri1.path).toBe("/foo/<bar>");
  expect(routeUri1.query).toBeUndefined();
  expect(routeUri1.asStr).toBe("/foo/<bar>");

  const routeUri2: RouteUriInfo = createRouteUri("/base", "/foo/<bar>?a=1");
  expect(routeUri2.base).toBe("/base");
  expect(routeUri2.unmountedOrigin).toBe("/foo/<bar>?a=1");
  expect(routeUri2.origin).toBe("/base/foo/<bar>?a=1");
  expect(routeUri2.path).toBe("/base/foo/<bar>");
  expect(routeUri2.query).toBe("a=1");
  expect(routeUri2.asStr).toBe("/base/foo/<bar>?a=1");
});

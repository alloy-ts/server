import { expect, test } from "vite-plus/test";
import { Route } from "../dist/index.js";

test("Route constructor computes default rank", () => {
  const indexRoute = new Route("GET", "/", "index", null);
  expect(indexRoute.method).toBe("GET");
  expect(indexRoute.uri).toBe("/");
  expect(indexRoute.name).toBe("index");
  expect(indexRoute.rank).toBe(-9);
});

test("Route ranked constructor and getters/setters", () => {
  const route = Route.ranked(2, "POST", "/route/<path..>?query", "route_name", "json");
  expect(route.name).toBe("route_name");
  expect(route.method).toBe("POST");
  expect(route.uri).toBe("/route/<path..>?query");
  expect(route.rank).toBe(2);
  expect(route.format).toBe("json");

  route.rank = 5;
  expect(route.rank).toBe(5);

  route.method = "PUT";
  expect(route.method).toBe("PUT");
});

test("Route mapBase prepends base prefix to URI", () => {
  const route = new Route("GET", "/foo/bar", null, null);
  expect(route.uri).toBe("/foo/bar");
  route.mapBase("/boo");
  expect(route.uri).toBe("/boo/foo/bar");
});

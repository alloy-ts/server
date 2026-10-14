import { expect, test } from "vite-plus/test";
import { Route } from "../dist/index.js";

test("Route computes default rank correctly", () => {
  const route = new Route("GET", "/foo/bar", "index", "json");
  expect(route.method).toBe("GET");
  expect(route.uri).toBe("/foo/bar");
  expect(route.rank).toBe(-9);
  expect(route.name).toBe("index");
  expect(route.format).toBe("application/json");
});

test("Route handles custom rank overrides", () => {
  const customRoute = Route.ranked(2, "POST", "/users", "create_user", "json");
  expect(customRoute.method).toBe("POST");
  expect(customRoute.uri).toBe("/users");
  expect(customRoute.rank).toBe(2);
  expect(customRoute.name).toBe("create_user");
  expect(customRoute.format).toBe("application/json");
});

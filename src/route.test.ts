import { expect, test } from "vite-plus/test";
import { Route } from "../dist/index.js";

test("Route constructor and fields", () => {
  const route = new Route("GET", "/users/<id>", "get_user", 1, "json");
  expect(route.method).toBe("GET");
  expect(route.uri).toBe("/users/<id>");
  expect(route.name).toBe("get_user");
  expect(route.rank).toBe(1);
  expect(route.format).toBe("json");
});

test("Route.ranked constructor", () => {
  const route = Route.ranked(2, "POST", "/posts");
  expect(route.method).toBe("POST");
  expect(route.uri).toBe("/posts");
  expect(route.rank).toBe(2);
});

test("Route mapBase", () => {
  const route = new Route("GET", "/foo/bar", "foo", 0, "json");
  route.mapBase("/api");
  expect(route.uri).toBe("/api/foo/bar");
});

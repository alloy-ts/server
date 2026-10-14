import { expect, test } from "vite-plus/test";
import { Route } from "../dist/index.js";

test("Route.create calculates default rank", () => {
  const index = Route.create("GET", "/");
  expect(index.rank).toBe(-9);
  expect(index.method).toBe("GET");
  expect(index.uri).toBe("/");
});

test("Route.ranked creates route with explicit or computed rank", () => {
  const custom = Route.ranked(2, "GET", "/route/<path..>?query");
  expect(custom.rank).toBe(2);
  expect(custom.method).toBe("GET");
  expect(custom.uri).toBe("/route/<path..>?query");

  const defaultRanked = Route.ranked(null, "POST", "/foo?bar");
  expect(defaultRanked.rank).toBe(-12);
  expect(defaultRanked.method).toBe("POST");
  expect(defaultRanked.uri).toBe("/foo?bar");
});

test("Route constructor supports full configuration", () => {
  const route = new Route("POST", "/users", "createUser", 1, "json");
  expect(route.name).toBe("createUser");
  expect(route.method).toBe("POST");
  expect(route.uri).toBe("/users");
  expect(route.rank).toBe(1);
  expect(route.format).toBe("json");
});

test("Route.mapBase maps the base URI", () => {
  const route = Route.create("GET", "/foo/bar");
  expect(route.uri).toBe("/foo/bar");

  route.mapBase("/api");
  expect(route.uri).toBe("/api/foo/bar");
});

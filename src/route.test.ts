import { expect, test } from "vite-plus/test";
import { Route } from "../dist/index.js";

test("Route constructor creates route with default rank computation", () => {
  const route = new Route("GET", "/foo/bar");
  expect(route.method).toBe("GET");
  expect(route.uri).toBe("/foo/bar");
  expect(route.rank).toBe(-9); // static path, no query
});

test("Route.ranked constructs route with explicit rank and method", () => {
  const route = Route.ranked(1, "POST", "/foo?bar");
  expect(route.method).toBe("POST");
  expect(route.uri).toBe("/foo?bar");
  expect(route.rank).toBe(1);

  const defaultRanked = Route.ranked(null, "GET", "/?foo");
  expect(defaultRanked.method).toBe("GET");
  expect(defaultRanked.rank).toBe(-12); // static path, static query
});

test("Route getters and setters update route metadata", () => {
  const route = new Route("GET", "/users");
  expect(route.name).toBeNull();

  route.name = "getUsers";
  expect(route.name).toBe("getUsers");

  route.method = "POST";
  expect(route.method).toBe("POST");

  route.rank = 5;
  expect(route.rank).toBe(5);

  route.format = "json";
  expect(route.format).toBe("application/json");

  route.format = "application/json";
  expect(route.format).toBe("application/json");

  route.uri = "/api/v1/users";
  expect(route.uri).toBe("/api/v1/users");
});

import { expect, test } from "vite-plus/test";
import { Route } from "../dist/index.js";

test("Route NAPI class creates routes and sets getters/setters", () => {
  const route = new Route("GET", "/api/items");
  expect(route.method).toBe("GET");
  expect(route.uri).toBe("/api/items");
  expect(route.rank).toBe(-9);
  expect(route.name).toBeNull();

  route.name = "get_items";
  expect(route.name).toBe("get_items");

  route.rank = 3;
  expect(route.rank).toBe(3);

  route.format = "json";
  expect(route.format).toBe("application/json");

  const rankedRoute = Route.ranked(2, "POST", "/submit");
  expect(rankedRoute.method).toBe("POST");
  expect(rankedRoute.rank).toBe(2);

  expect(() => new Route("INVALID", "///")).toThrow();
});

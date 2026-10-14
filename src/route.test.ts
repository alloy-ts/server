import { expect, test } from "vite-plus/test";
import { createRoute, Alloy } from "./main.ts";

test("createRoute creates valid Route object metadata", () => {
  const route = createRoute("get", "/route/<path..>?query", "route_name", 2, "json");
  expect(route.name).toBe("route_name");
  expect(route.method).toBe("GET");
  expect(route.uri).toBe("/route/<path..>?query");
  expect(route.rank).toBe(2);
  expect(route.format).toBe("json");
});

test("createRoute assigns default rank when rank is omitted", () => {
  const route = Alloy.createRoute("post", "/items");
  expect(route.method).toBe("POST");
  expect(route.uri).toBe("/items");
  expect(route.rank).toBe(-9);
  expect(route.name).toBeUndefined();
  expect(route.format).toBeUndefined();
});

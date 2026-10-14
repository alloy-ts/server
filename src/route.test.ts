import { expect, test } from "vite-plus/test";
import { RocketRoute } from "./main.ts";

test("RocketRoute.new creates route with computed default rank", () => {
  const route = new RocketRoute("GET", "/");
  expect(route.method).toBe("GET");
  expect(route.uri).toBe("/");
  expect(route.rank).toBe(-9);
});

test("RocketRoute.ranked creates route with explicit rank", () => {
  const route = RocketRoute.ranked(2, "GET", "/route/<path..>?query");
  expect(route.method).toBe("GET");
  expect(route.uri).toBe("/route/<path..>?query");
  expect(route.rank).toBe(2);
});

test("RocketRoute computes correct default ranks for queries", () => {
  const route = new RocketRoute("GET", "/?foo");
  expect(route.method).toBe("GET");
  expect(route.uri).toBe("/?foo");
  expect(route.rank).toBe(-12);
});

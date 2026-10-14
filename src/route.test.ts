import { expect, test } from "vite-plus/test";
import { Route } from "../dist/index.js";

test("Route constructor creates route with default rank", () => {
  const route = new Route("GET", "/hello");
  expect(route.method).toBe("GET");
  expect(route.uri).toBe("/hello");
  expect(route.rank).toBe(-9);
});

test("Route constructor for dynamic path computes rank", () => {
  const route = new Route("GET", "/hello/<name>");
  expect(route.rank).toBe(-5);
});

test("Route.ranked creates route with explicit rank", () => {
  const route = Route.ranked(5, "POST", "/submit");
  expect(route.method).toBe("POST");
  expect(route.uri).toBe("/submit");
  expect(route.rank).toBe(5);
});

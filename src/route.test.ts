import { expect, test } from "vite-plus/test";
import { Route } from "../dist/index.js";

test("Route creates route with calculated default rank", () => {
  const r1 = new Route("GET", "/foo/bar");
  expect(r1.method).toBe("GET");
  expect(r1.uri).toBe("/foo/bar");
  expect(r1.rank).toBe(-9); // static path, no query
  expect(r1.base).toBe("/");
});

test("Route.ranked constructs route with specified rank", () => {
  const r2 = Route.ranked(2, "POST", "/submit?query");
  expect(r2.rank).toBe(2);
  expect(r2.method).toBe("POST");
  expect(r2.uri).toBe("/submit?query");
});

test("Route map_base modifies route base URI", () => {
  const r3 = new Route("GET", "/api/item");
  r3.mapBase("/v1");
  expect(r3.base).toBe("/v1");
});

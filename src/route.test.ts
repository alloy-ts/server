import { expect, test } from "vite-plus/test";
import { Route } from "../dist/index.js";

test("Route constructor creates route with default rank", () => {
  const index = new Route("GET", "/");
  expect(index.method).toBe("GET");
  expect(index.uri).toBe("/");
  expect(index.rank).toBe(-9);
  expect(index.name).toBeNull();
  expect(index.format).toBeNull();
});

test("Route computes default rank according to static/dynamic path and query table", () => {
  // static path, static query -> -12
  const staticQuery = new Route("GET", "/foo/bar?a=b&bob");
  expect(staticQuery.rank).toBe(-12);

  // static path, partial query -> -11
  const partialQuery = new Route("GET", "/?a&<zoo..>");
  expect(partialQuery.rank).toBe(-11);

  // static path, wild query -> -10
  const wildQuery = new Route("GET", "/?<zoo..>");
  expect(wildQuery.rank).toBe(-10);

  // static path, no query -> -9
  const staticPath = new Route("GET", "/foo/bar");
  expect(staticPath.rank).toBe(-9);

  // partial path, no query -> -5
  const partialPath = new Route("GET", "/a/<b>");
  expect(partialPath.rank).toBe(-5);

  // wild path, no query -> -1
  const wildPath = new Route("GET", "/<b>/<c>");
  expect(wildPath.rank).toBe(-1);
});

test("Route.ranked creates route with explicit or default rank", () => {
  const explicit = Route.ranked(2, "POST", "/route/<path..>?query");
  expect(explicit.rank).toBe(2);
  expect(explicit.method).toBe("POST");
  expect(explicit.uri).toBe("/route/<path..>?query");

  const computed = Route.ranked(null, "POST", "/foo?bar");
  expect(computed.rank).toBe(-12);
});

test("Route getters and setters update properties accurately", () => {
  const route = new Route("GET", "/items");

  route.name = "get_items";
  expect(route.name).toBe("get_items");

  route.method = "POST";
  expect(route.method).toBe("POST");

  route.rank = 5;
  expect(route.rank).toBe(5);

  route.format = "json";
  expect(route.format).toBe("json");
});

test("Route validates method and format inputs", () => {
  expect(() => new Route("INVALID_METHOD", "/")).toThrow(/invalid HTTP method/);

  const route = new Route("GET", "/");
  expect(() => {
    route.format = "invalid/format/type!";
  }).toThrow();
});

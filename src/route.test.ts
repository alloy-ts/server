import { expect, test } from "vite-plus/test";
import { Route } from "../dist/index.js";

test("Route computes default rank according to Rocket ranking table", () => {
  const assertRank = (uri: string, expectedRank: number) => {
    const route = new Route("GET", uri);
    expect(route.rank).toBe(expectedRank);
  };

  // Static path, static query -> -12
  assertRank("/?foo", -12);
  assertRank("/foo/bar?a=b&bob", -12);

  // Static path, partial query -> -11
  assertRank("/?a&<zoo..>", -11);
  assertRank("/foo?a&<zoo>", -11);

  // Static path, wild query -> -10
  assertRank("/?<zoo..>", -10);
  assertRank("/foo?<a>&<b>", -10);

  // Static path, no query -> -9
  assertRank("/", -9);
  assertRank("/foo/bar", -9);

  // Partial path, static query -> -8
  assertRank("/a/<b>?foo", -8);

  // Partial path, partial query -> -7
  assertRank("/a/<b>?<b>&c", -7);

  // Partial path, wild query -> -6
  assertRank("/a/<b>?<c..>", -6);

  // Partial path, no query -> -5
  assertRank("/a/<b>", -5);

  // Wild path, static query -> -4
  assertRank("/<b>/<c>?foo&bar", -4);

  // Wild path, partial query -> -3
  assertRank("/<b>/<c>?<foo>&bar", -3);

  // Wild path, wild query -> -2
  assertRank("/<b>/<c>?<foo>", -2);

  // Wild path, no query -> -1
  assertRank("/<b>/<c>", -1);
  assertRank("/<b..>", -1);
});

test("Route.ranked creates route with explicit rank and format", () => {
  const route = Route.ranked(1, "POST", "/foo?bar", "route_name", "json");
  expect(route.rank).toBe(1);
  expect(route.method).toBe("POST");
  expect(route.uri).toBe("/foo?bar");
  expect(route.name).toBe("route_name");
  expect(route.format).toBe("application/json");
});

test("Route.mapBase maps route base uri using string or callback", () => {
  const route = new Route("GET", "/foo/bar");
  expect(route.uri).toBe("/foo/bar");

  const mapped = route.mapBase((base: string) => `/boo${base}`);
  expect(mapped.uri).toBe("/boo/foo/bar");

  const directMapped = route.mapBase("/api");
  expect(directMapped.uri).toBe("/api/foo/bar");
});

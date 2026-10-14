import { expect, test } from "vite-plus/test";
import { Route, computeDefaultRank } from "./main.ts";

test("computeDefaultRank computes correct Rocket default ranks", () => {
  expect(computeDefaultRank("/?foo")).toBe(-12); // static path, static query
  expect(computeDefaultRank("/foo/bar?a=b&bob")).toBe(-12);
  expect(computeDefaultRank("/?a=b&bob")).toBe(-12);

  expect(computeDefaultRank("/?a&<zoo..>")).toBe(-11); // static path, partial query
  expect(computeDefaultRank("/foo?a&<zoo..>")).toBe(-11);
  expect(computeDefaultRank("/?a&<zoo>")).toBe(-11);

  expect(computeDefaultRank("/?<zoo..>")).toBe(-10); // static path, wild query
  expect(computeDefaultRank("/foo?<zoo..>")).toBe(-10);

  expect(computeDefaultRank("/")).toBe(-9); // static path, no query
  expect(computeDefaultRank("/foo/bar")).toBe(-9);

  expect(computeDefaultRank("/a/<b>?foo")).toBe(-8); // partial path, static query
  expect(computeDefaultRank("/a/<b>?<b>&c")).toBe(-7); // partial path, partial query
  expect(computeDefaultRank("/a/<b>?<c..>")).toBe(-6); // partial path, wild query
  expect(computeDefaultRank("/a/<b>")).toBe(-5); // partial path, no query

  expect(computeDefaultRank("/<b>/<c>?foo&bar")).toBe(-4); // wild path, static query
  expect(computeDefaultRank("/<b>/<c>?<foo>&bar")).toBe(-3); // wild path, partial query
  expect(computeDefaultRank("/<b>/<c>?<foo>")).toBe(-2); // wild path, wild query
  expect(computeDefaultRank("/<b>/<c>")).toBe(-1); // wild path, no query
});

test("Route class initialization and ranked constructor", () => {
  const route1 = new Route("GET", "/foo/bar", "route_name", "json");
  expect(route1.method).toBe("GET");
  expect(route1.uri).toBe("/foo/bar");
  expect(route1.rank).toBe(-9);
  expect(route1.name).toBe("route_name");
  expect(route1.format).toBe("json");

  const route2 = Route.ranked(2, "POST", "/route/<path..>?query");
  expect(route2.rank).toBe(2);
  expect(route2.method).toBe("POST");
  expect(route2.uri).toBe("/route/<path..>?query");

  const route3 = Route.ranked(null, "GET", "/?foo");
  expect(route3.rank).toBe(-12);
});

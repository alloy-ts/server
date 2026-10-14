import { expect, test } from "vite-plus/test";
import { Route } from "../dist/index.js";

test("Route constructor and ranked factory", () => {
  const route = new Route("GET", "/", "index_route", "json");
  expect(route.method).toBe("GET");
  expect(route.uri).toBe("/");
  expect(route.name).toBe("index_route");
  expect(route.format).toBe("application/json");
  expect(route.rank).toBe(-9);

  const customRank = Route.ranked(2, "POST", "/route/<path..>?query", "route_name", "json");
  expect(customRank.rank).toBe(2);
  expect(customRank.method).toBe("POST");
  expect(customRank.uri).toBe("/route/<path..>?query");
  expect(customRank.name).toBe("route_name");
  expect(customRank.format).toBe("application/json");
});

test("Route calculates default ranks matching Rocket rank table", () => {
  // static path, static query -> -12
  expect(Route.ranked(null, "GET", "/?foo").rank).toBe(-12);
  expect(Route.ranked(null, "GET", "/foo/bar?a=b&bob").rank).toBe(-12);

  // static path, partial query -> -11
  expect(Route.ranked(null, "GET", "/?a&<zoo..>").rank).toBe(-11);

  // static path, wild query -> -10
  expect(Route.ranked(null, "GET", "/?<zoo..>").rank).toBe(-10);

  // static path, no query -> -9
  expect(Route.ranked(null, "GET", "/").rank).toBe(-9);
  expect(Route.ranked(null, "GET", "/foo/bar").rank).toBe(-9);

  // partial path, static query -> -8
  expect(Route.ranked(null, "GET", "/a/<b>?foo").rank).toBe(-8);

  // partial path, partial query -> -7
  expect(Route.ranked(null, "GET", "/a/<b>?<b>&c").rank).toBe(-7);

  // partial path, wild query -> -6
  expect(Route.ranked(null, "GET", "/a/<b>?<c..>").rank).toBe(-6);

  // partial path, no query -> -5
  expect(Route.ranked(null, "GET", "/a/<b>").rank).toBe(-5);

  // wild path, static query -> -4
  expect(Route.ranked(null, "GET", "/<b>/<c>?foo&bar").rank).toBe(-4);

  // wild path, partial query -> -3
  expect(Route.ranked(null, "GET", "/<b>/<c>?<foo>&bar").rank).toBe(-3);

  // wild path, wild query -> -2
  expect(Route.ranked(null, "GET", "/<b>/<c>?<foo>").rank).toBe(-2);

  // wild path, no query -> -1
  expect(Route.ranked(null, "GET", "/<b>/<c>").rank).toBe(-1);
});

test("Route mapBase updates mounted base URI", () => {
  const route = new Route("GET", "/foo/bar");
  expect(route.uri).toBe("/foo/bar");

  const newUri = route.mapBase("/boo");
  expect(newUri).toBe("/boo/foo/bar");
  expect(route.uri).toBe("/boo/foo/bar");
});

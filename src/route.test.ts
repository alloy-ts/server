import { expect, test } from "vite-plus/test";
import { type RouteInfo, computeDefaultRank, createRouteInfo } from "../dist/index.js";

test("Route default rank calculation according to Rocket ranking table", () => {
  expect(computeDefaultRank("/foo/bar?a=b&bob")).toBe(-12);
  expect(computeDefaultRank("/foo?a&<zoo..>")).toBe(-11);
  expect(computeDefaultRank("/foo?<zoo..>")).toBe(-10);
  expect(computeDefaultRank("/foo/bar")).toBe(-9);
  expect(computeDefaultRank("/")).toBe(-9);
  expect(computeDefaultRank("/a/<b>?foo")).toBe(-8);
  expect(computeDefaultRank("/a/<b>?a&<c..>")).toBe(-7);
  expect(computeDefaultRank("/a/<b>?<c..>")).toBe(-6);
  expect(computeDefaultRank("/a/<b>")).toBe(-5);
  expect(computeDefaultRank("/<b>/<c>?foo&bar")).toBe(-4);
  expect(computeDefaultRank("/<b>/<c>?a&<b..>")).toBe(-3);
  expect(computeDefaultRank("/<b>/<c>?<foo>")).toBe(-2);
  expect(computeDefaultRank("/<b>/<c>")).toBe(-1);
  expect(computeDefaultRank("/<..>")).toBe(-1);
});

test("createRouteInfo constructor helper", () => {
  const info: RouteInfo = createRouteInfo("get", "/users/<id>", "user_route", undefined, "json");

  expect(info.name).toBe("user_route");
  expect(info.method).toBe("GET");
  expect(info.uri).toBe("/users/<id>");
  expect(info.rank).toBe(-5);
  expect(info.format).toBe("json");
});

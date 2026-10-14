import { expect, test } from "vite-plus/test";
import {
  type HandlerOutcome,
  type OutcomeKind,
  type RouteInfo,
  computeDefaultRank,
  createRouteInfo,
} from "../dist/index.js";

test("OutcomeKind and HandlerOutcome NAPI bindings", () => {
  const outcome: HandlerOutcome = {
    kind: 0 as OutcomeKind, // OutcomeKind.Success
    status: 200,
    body: [72, 101, 108, 108, 111], // "Hello"
    headers: { "Content-Type": "text/plain" },
  };

  expect(outcome.kind).toBe(0);
  expect(outcome.status).toBe(200);
  expect(outcome.body).toHaveLength(5);
  expect(outcome.headers?.["Content-Type"]).toBe("text/plain");
});

test("Route default rank calculation according to Rocket ranking table", () => {
  // Static path, static query -> -12
  expect(computeDefaultRank("/foo/bar?a=b&bob")).toBe(-12);

  // Static path, partial query -> -11
  expect(computeDefaultRank("/foo?a&<zoo..>")).toBe(-11);

  // Static path, wild query -> -10
  expect(computeDefaultRank("/foo?<zoo..>")).toBe(-10);

  // Static path, no query -> -9
  expect(computeDefaultRank("/foo/bar")).toBe(-9);
  expect(computeDefaultRank("/")).toBe(-9);

  // Partial path, static query -> -8
  expect(computeDefaultRank("/a/<b>?foo")).toBe(-8);

  // Partial path, partial query -> -7
  expect(computeDefaultRank("/a/<b>?a&<c..>")).toBe(-7);

  // Partial path, wild query -> -6
  expect(computeDefaultRank("/a/<b>?<c..>")).toBe(-6);

  // Partial path, no query -> -5
  expect(computeDefaultRank("/a/<b>")).toBe(-5);

  // Wild path, static query -> -4
  expect(computeDefaultRank("/<b>/<c>?foo&bar")).toBe(-4);

  // Wild path, partial query -> -3
  expect(computeDefaultRank("/<b>/<c>?a&<b..>")).toBe(-3);

  // Wild path, wild query -> -2
  expect(computeDefaultRank("/<b>/<c>?<foo>")).toBe(-2);

  // Wild path, no query -> -1
  expect(computeDefaultRank("/<b>/<c>")).toBe(-1);
  expect(computeDefaultRank("/<..>")).toBe(-1);
});

test("createRouteInfo constructor helper", () => {
  const info: RouteInfo = createRouteInfo("get", "/users/<id>", "user_route", undefined, "json");

  expect(info.name).toBe("user_route");
  expect(info.method).toBe("GET");
  expect(info.uri).toBe("/users/<id>");
  expect(info.rank).toBe(-5); // Computed default rank for partial path /users/<id>
  expect(info.format).toBe("json");
});

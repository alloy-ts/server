import { expect, test } from "vite-plus/test";
import { Route } from "../dist/index.js";

test("Route constructor creates new route with default rank", () => {
  const index = new Route("GET", "/");
  expect(index.rank).toBe(-9);
  expect(index.method).toBe("GET");
  expect(index.uri).toBe("/");
});

test("Route.ranked creates route with explicit or default rank", () => {
  const customRank = Route.ranked(1, "POST", "/foo?bar");
  expect(customRank.rank).toBe(1);
  expect(customRank.method).toBe("POST");
  expect(customRank.uri).toBe("/foo?bar");

  const defaultRank = Route.ranked(null, "POST", "/foo?bar");
  expect(defaultRank.rank).toBe(-12);
  expect(defaultRank.method).toBe("POST");
  expect(defaultRank.uri).toBe("/foo?bar");
});

test("Route default ranking table matches Rocket defaults", () => {
  expect(new Route("GET", "/?foo").rank).toBe(-12);
  expect(new Route("GET", "/?a&<zoo..>").rank).toBe(-11);
  expect(new Route("GET", "/?<zoo..>").rank).toBe(-10);
  expect(new Route("GET", "/").rank).toBe(-9);
  expect(new Route("GET", "/a/<b>?foo").rank).toBe(-8);
  expect(new Route("GET", "/a/<b>").rank).toBe(-5);
  expect(new Route("GET", "/<b>/<c>").rank).toBe(-1);
});

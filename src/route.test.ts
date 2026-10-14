import { expect, test } from "vite-plus/test";
import { Route } from "../index.js";

test("route.ts - Route.new default rank calculation", () => {
  const index = new Route("GET", "/");
  expect(index.method).toBe("GET");
  expect(index.uri).toBe("/");
  expect(index.rank).toBe(-9);

  const staticQuery = new Route("GET", "/foo/bar?a=b");
  expect(staticQuery.method).toBe("GET");
  expect(staticQuery.uri).toBe("/foo/bar?a=b");
  expect(staticQuery.rank).toBe(-12);
});

test("route.ts - Route.ranked explicit rank", () => {
  const foo = Route.ranked(1, "POST", "/foo?bar");
  expect(foo.rank).toBe(1);
  expect(foo.method).toBe("POST");
  expect(foo.uri).toBe("/foo?bar");

  const fooDefault = Route.ranked(null, "POST", "/foo?bar");
  expect(fooDefault.rank).toBe(-12);
  expect(fooDefault.method).toBe("POST");
});

test("route.ts - Route mapBase", () => {
  const index = new Route("GET", "/foo/bar");
  expect(index.uri).toBe("/foo/bar");

  const mapped = index.mapBase("/boo");
  expect(mapped.uri).toBe("/boo/foo/bar");
});

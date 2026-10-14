import { expect, test } from "vite-plus/test";
import { Route } from "./main.ts";

test("Route default rank matches Rocket ranking table", () => {
  const r1 = new Route("GET", "/?foo");
  expect(r1.rank).toBe(-12);

  const r2 = new Route("GET", "/?a&<zoo..>");
  expect(r2.rank).toBe(-11);

  const r3 = new Route("GET", "/?<zoo..>");
  expect(r3.rank).toBe(-10);

  const r4 = new Route("GET", "/");
  expect(r4.rank).toBe(-9);

  const r5 = new Route("GET", "/a/<b>?foo");
  expect(r5.rank).toBe(-8);

  const r6 = new Route("GET", "/a/<b>?<b>&c");
  expect(r6.rank).toBe(-7);

  const r7 = new Route("GET", "/a/<b>?<c..>");
  expect(r7.rank).toBe(-6);

  const r8 = new Route("GET", "/a/<b>");
  expect(r8.rank).toBe(-5);

  const r9 = new Route("GET", "/<b>/<c>?foo&bar");
  expect(r9.rank).toBe(-4);

  const r10 = new Route("GET", "/<b>/<c>?<foo>&bar");
  expect(r10.rank).toBe(-3);

  const r11 = new Route("GET", "/<b>/<c>?<foo>");
  expect(r11.rank).toBe(-2);

  const r12 = new Route("GET", "/<b>/<c>");
  expect(r12.rank).toBe(-1);
});

test("Route fields and ranked constructor", () => {
  const route1 = new Route("get", "/route/<path..>?query", "route_name", "json");
  expect(route1.name).toBe("route_name");
  expect(route1.method).toBe("GET");
  expect(route1.uri).toBe("/route/<path..>?query");
  expect(route1.format).toBe("json");

  const route2 = Route.ranked(2, "get", "/route/<path..>?query", "route_name", "json");
  expect(route2.rank).toBe(2);
  expect(route2.name).toBe("route_name");
});

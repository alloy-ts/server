import { expect, test } from "vite-plus/test";
import { Route } from "../dist/index.js";

test("Route NAPI class constructs route and calculates default rank", () => {
  const r1 = new Route("GET", "/");
  expect(r1.method).toBe("GET");
  expect(r1.uri).toBe("/");
  expect(r1.rank).toBe(-9);

  const r2 = new Route("GET", "/?foo");
  expect(r2.rank).toBe(-12);

  const r3 = Route.ranked(2, "POST", "/route/<path..>?query");
  expect(r3.rank).toBe(2);
  expect(r3.method).toBe("POST");
  expect(r3.uri).toBe("/route/<path..>?query");
});

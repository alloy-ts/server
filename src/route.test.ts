import { expect, test } from "vite-plus/test";
import { Route } from "../index.js";

test("Route class initialization and computed default rank", () => {
  const r = new Route("GET", "/hello/<name>", "hello");
  expect(r.method).toBe("GET");
  expect(r.uri).toBe("/hello/<name>");
  expect(r.name).toBe("hello");
  expect(typeof r.rank).toBe("number");

  const ranked = Route.ranked(-5, "POST", "/submit", "submit");
  expect(ranked.rank).toBe(-5);
  expect(ranked.method).toBe("POST");
});

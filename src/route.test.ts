import { expect, test } from "vite-plus/test";
import { Route } from "./main.ts";

test("Route class initialization and computed default rank", () => {
  const r1 = new Route("GET", "/", "index");
  expect(r1.method).toBe("GET");
  expect(r1.uri).toBe("/");
  expect(r1.name).toBe("index");
  expect(r1.rank).toBe(-9);

  const r2 = Route.ranked(1, "POST", "/users", "create_user");
  expect(r2.method).toBe("POST");
  expect(r2.rank).toBe(1);
});

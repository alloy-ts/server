import { expect, test } from "vite-plus/test";
import { Route } from "../dist/index.js";

test("Route computes default rank and handles explicit rank", () => {
  const index = new Route("GET", "/");
  expect(index.method).toBe("GET");
  expect(index.uri).toBe("/");
  expect(index.rank).toBe(-9);

  const staticQuery = new Route("GET", "/?foo");
  expect(staticQuery.rank).toBe(-12);

  const explicitRank = Route.ranked(2, "GET", "/route/<path..>?query");
  expect(explicitRank.method).toBe("GET");
  expect(explicitRank.uri).toBe("/route/<path..>?query");
  expect(explicitRank.rank).toBe(2);
});

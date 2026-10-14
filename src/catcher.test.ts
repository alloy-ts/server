import { expect, test } from "vite-plus/test";
import { Catcher } from "../dist/index.js";

test("Catcher initializes with status code and default base", () => {
  const c1 = new Catcher(404, "not_found");
  expect(c1.code).toBe(404);
  expect(c1.name).toBe("not_found");
  expect(c1.base).toBe("/");
});

test("Catcher supports null/undefined code for default catchers", () => {
  const c2 = new Catcher(null, "default_catcher");
  expect(c2.code).toBeNull();
  expect(c2.name).toBe("default_catcher");
});

test("Catcher throws error for invalid HTTP error status codes", () => {
  expect(() => new Catcher(200, "invalid")).toThrow();
  expect(() => new Catcher(600, "invalid")).toThrow();
});

test("Catcher mapBase updates base URI", () => {
  const c3 = new Catcher(500, "internal_error");
  c3.mapBase("/api");
  expect(c3.base).toBe("/api");
});

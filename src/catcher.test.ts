import { expect, test } from "vite-plus/test";
import { Catcher } from "../dist/index.js";

test("Catcher.forCode creates Catcher for HTTP error status", () => {
  const notFound = Catcher.forCode(404, "not_found");
  expect(notFound.code).toBe(404);
  expect(notFound.name).toBe("not_found");
  expect(notFound.base).toBe("/");
});

test("Catcher.defaultCatcher creates default catcher", () => {
  const defaultCatcher = Catcher.defaultCatcher("default_catcher");
  expect(defaultCatcher.code).toBeFalsy();
  expect(defaultCatcher.name).toBe("default_catcher");
  expect(defaultCatcher.base).toBe("/");
});

test("Catcher constructor validates HTTP status range [400, 600)", () => {
  expect(() => new Catcher(200, "invalid")).toThrow(/must be in the HTTP error range/);
  expect(() => new Catcher(600, "invalid")).toThrow(/must be in the HTTP error range/);

  const valid = new Catcher(500, "internal_error", "/api");
  expect(valid.code).toBe(500);
  expect(valid.base).toBe("/api");
});

test("Catcher.mapBase maps base URI", () => {
  const catcher = Catcher.forCode(404, "not_found");
  expect(catcher.base).toBe("/");

  catcher.mapBase("/error");
  expect(catcher.base).toBe("/error");
});

import { expect, test } from "vite-plus/test";
import { Catcher } from "../dist/index.js";

test("Catcher creates valid status code catchers and validates status range", () => {
  const notFound = new Catcher(404, "/");
  expect(notFound.code).toBe(404);
  expect(notFound.base).toBe("/");
  expect(notFound.toString()).toBe("/404 ");

  const internalError = new Catcher(500, "/api");
  expect(internalError.code).toBe(500);
  expect(internalError.base).toBe("/api");

  expect(() => new Catcher(200, "/")).toThrow(/error range/);
  expect(() => new Catcher(600, "/")).toThrow(/error range/);
});

test("Catcher supports default catchers", () => {
  const defaultCatcher = Catcher.defaultCatcher("/v1");
  expect(defaultCatcher.code).toBeNull();
  expect(defaultCatcher.base).toBe("/v1");
  expect(defaultCatcher.toString()).toContain("default");
});

test("Catcher mapBase transforms base URI path", () => {
  const catcher = new Catcher(400, "/bar");
  expect(catcher.base).toBe("/bar");

  catcher.mapBase("/foo/bar");
  expect(catcher.base).toBe("/foo/bar");

  expect(() => catcher.mapBase("invalid uri path")).toThrow();
});

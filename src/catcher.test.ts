import { expect, test } from "vite-plus/test";
import { createCatcher } from "../dist/index.js";

test("createCatcher creates Catcher metadata object for status codes", () => {
  const catcher = createCatcher(404, "not_found", "/api");
  expect(catcher.code).toBe(404);
  expect(catcher.name).toBe("not_found");
  expect(catcher.base).toBe("/api");
});

test("createCatcher handles default catcher without status code", () => {
  const catcher = createCatcher(undefined, "default_catcher");
  expect(catcher.code).toBeUndefined();
  expect(catcher.name).toBe("default_catcher");
  expect(catcher.base).toBe("/");
});

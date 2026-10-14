import { expect, test } from "vite-plus/test";
import { Catcher } from "../dist/index.js";

test("Catcher instantiation, code validation, and properties", () => {
  const c404 = new Catcher(404, "/", "not_found");
  expect(c404.code).toBe(404);
  expect(c404.base).toBe("/");
  expect(c404.name).toBe("not_found");
  expect(c404.isDefault).toBe(false);

  const defaultCatcher = new Catcher(null, "/api", "default_handler");
  expect(defaultCatcher.code).toBeNull();
  expect(defaultCatcher.base).toBe("/api");
  expect(defaultCatcher.isDefault).toBe(true);

  expect(() => new Catcher(200, "/")).toThrow();
  expect(() => new Catcher(600, "/")).toThrow();
});

test("Catcher mapBase updates mount point", () => {
  const catcher = new Catcher(500, "/");
  expect(catcher.base).toBe("/");

  catcher.mapBase("/app");
  expect(catcher.base).toBe("/app");
});

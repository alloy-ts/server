import { expect, test } from "vite-plus/test";
import { Catcher } from "../index.js";

test("catcher.ts - Catcher creation with status code", () => {
  const c404 = new Catcher(404);
  expect(c404.code).toBe(404);
  expect(c404.base).toBe("/");

  const c500 = new Catcher(500);
  expect(c500.code).toBe(500);
});

test("catcher.ts - Catcher default_catcher static constructor", () => {
  const def = Catcher.defaultCatcher();
  expect(def.code).toBeNull();
  expect(def.base).toBe("/");
  expect(def.name).toBe("<Rocket Catcher>");
});

test("catcher.ts - Catcher map_base updates base path", () => {
  const c = new Catcher(404);
  expect(c.base).toBe("/");

  const updatedBase = c.mapBase("/api/v1");
  expect(updatedBase).toBe("/api/v1");
  expect(c.base).toBe("/api/v1");
});

test("catcher.ts - Catcher invalid status code throws error", () => {
  expect(() => new Catcher(200)).toThrow();
  expect(() => new Catcher(600)).toThrow();
});

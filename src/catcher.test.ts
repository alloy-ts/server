import { expect, test } from "vite-plus/test";
import { Catcher } from "../dist/index.js";

test("Catcher creation, code, name, base, and mapBase", () => {
  const c1 = new Catcher(404, "not_found", "/api");
  expect(c1.code).toBe(404);
  expect(c1.name).toBe("not_found");
  expect(c1.base).toBe("/api");

  c1.mapBase("/v1/api");
  expect(c1.base).toBe("/v1/api");

  const defaultCatcher = new Catcher(null, "default_catcher");
  expect(defaultCatcher.code).toBeNull();
  expect(defaultCatcher.name).toBe("default_catcher");
  expect(defaultCatcher.base).toBe("/");
});

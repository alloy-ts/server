import { expect, test } from "vite-plus/test";
import { Catcher } from "../index.js";

test("Catcher class initialization and base mapping", () => {
  const catcher = new Catcher(404, "not_found_catcher");
  expect(catcher.code).toBe(404);
  expect(catcher.name).toBe("not_found_catcher");
  expect(catcher.base).toBe("/");

  catcher.mapBase("/api");
  expect(catcher.base).toBe("/api");
});

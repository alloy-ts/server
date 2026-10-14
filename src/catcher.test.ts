import { expect, test } from "vite-plus/test";
import { Catcher } from "../dist/index.js";

test("Catcher constructor and properties", () => {
  const catcher = new Catcher(404, "not_found");
  expect(catcher.code).toBe(404);
  expect(catcher.name).toBe("not_found");
  expect(catcher.base).toBe("/");
});

test("Catcher mapBase", () => {
  const catcher = new Catcher(500, "error");
  catcher.mapBase("/api");
  expect(catcher.base).toBe("/api/");
});

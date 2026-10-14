import { describe, expect, test } from "vite-plus/test";
import { Catcher } from "../dist/index.js";

describe("Catcher NAPI class", () => {
  test("creates status code catcher", () => {
    const catcher = new Catcher(404);
    expect(catcher.code).toBe(404);
    expect(catcher.base).toBe("/");
  });

  test("creates default catcher with null/undefined code", () => {
    const catcher = new Catcher();
    expect(catcher.code).toBeNull();
    expect(catcher.base).toBe("/");
  });

  test("maps base path using mapBase", () => {
    const catcher = new Catcher(500);
    const rebased = catcher.mapBase("/api");
    expect(rebased.base).toBe("/api");
  });
});

import { describe, it, expect } from "vite-plus/test";
import { Catcher } from "../dist/index.js";

describe("Catcher NAPI bindings", () => {
  it("should create a Catcher for status code 404", () => {
    const catcher = new Catcher(404, "not_found");
    expect(catcher.code).toBe(404);
    expect(catcher.name).toBe("not_found");
    expect(catcher.base).toBe("/");
    expect(catcher.isDefault).toBe(false);
  });

  it("should create a default Catcher", () => {
    const catcher = new Catcher(null, "default_catcher");
    expect(catcher.code).toBeNull();
    expect(catcher.name).toBe("default_catcher");
    expect(catcher.base).toBe("/");
    expect(catcher.isDefault).toBe(true);
  });

  it("should map base correctly", () => {
    const catcher = new Catcher(500, "internal_error");
    expect(catcher.base).toBe("/");
    const newBase = catcher.mapBase("/api");
    expect(newBase).toBe("/api");
    expect(catcher.base).toBe("/api");
  });

  it("should throw error for status code outside [400, 600)", () => {
    expect(() => new Catcher(200, "ok_catcher")).toThrow("Status code must be in range [400, 600)");
  });
});

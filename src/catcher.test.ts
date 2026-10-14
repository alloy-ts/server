import { describe, expect, it } from "vite-plus/test";
import { Catcher } from "../dist/index.js";

describe("Catcher NAPI Bindings", () => {
  it("creates a Catcher with status code 404", () => {
    const catcher = Catcher.status(404);
    expect(catcher.code).toBe(404);
    expect(catcher.base).toBe("/");
  });

  it("creates a Catcher with status code 500", () => {
    const catcher = Catcher.status(500);
    expect(catcher.code).toBe(500);
  });

  it("creates a default Catcher", () => {
    const catcher = Catcher.defaultCatcher();
    expect(catcher.code).toBeNull();
    expect(catcher.name).toBe("default");
  });

  it("throws for status codes outside [400, 600)", () => {
    expect(() => new Catcher(200)).toThrow();
  });
});

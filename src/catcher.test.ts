import { expect, test, describe } from "vite-plus/test";
import { Catcher, Handler, response } from "../dist/index.js";

describe("Rocket Catcher NAPI bindings", () => {
  describe("Catcher constructor and property getters", () => {
    test("creates Catcher for 404 status code", () => {
      const handler = new Handler(async () => response("404 Not Found", { status: 404 }));
      const catcher = new Catcher(404, handler);

      expect(catcher).toBeInstanceOf(Catcher);
      expect(catcher.code).toBe(404);
      expect(catcher.base).toBe("/");
    });

    test("creates default Catcher when code is null or undefined", () => {
      const handler = new Handler(async () => response("Default Error Handler", { status: 500 }));
      const catcher = new Catcher(null, handler);

      expect(catcher.code).toBeNull();
      expect(catcher.base).toBe("/");
    });

    test("Catcher.defaultCatcher creates Rocket built-in default catcher", () => {
      const catcher = Catcher.defaultCatcher();
      expect(catcher).toBeInstanceOf(Catcher);
      expect(catcher.code).toBeNull();
    });
  });

  describe("mapBase and toString methods", () => {
    test("mapBase maps the base mount point of a catcher", () => {
      const handler = new Handler(async () => response("Not Found", { status: 404 }));
      const catcher = new Catcher(404, handler);
      expect(catcher.base).toBe("/");

      const mapped = catcher.mapBase("/bar");
      expect(mapped.base).toBe("/bar");
    });

    test("toString returns string representation of the catcher", () => {
      const handler = new Handler(async () => response("Not Found", { status: 404 }));
      const catcher = new Catcher(404, handler);
      const str = catcher.toString();

      expect(str).toContain("404");
    });
  });

  describe("error status code validation", () => {
    test("throws error when status code is outside [400, 600) HTTP error range", () => {
      const handler = new Handler(async () => response("OK"));

      expect(() => new Catcher(200, handler)).toThrow(/outside HTTP error range/);
      expect(() => new Catcher(600, handler)).toThrow(/outside HTTP error range/);
    });
  });
});

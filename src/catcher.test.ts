import { describe, expect, it } from "vite-plus/test";
import { Catcher } from "../dist/index.js";

describe("Catcher NAPI bindings", () => {
  it("instantiates Catcher for 404 status code", () => {
    const catcher = new Catcher(404, async () => {
      return { status: 404, body: "Not Found Custom" };
    });

    expect(catcher).toBeDefined();
    expect(catcher.code).toBe(404);
    expect(catcher.base).toBe("/");
  });

  it("instantiates default Catcher when code is null/undefined", () => {
    const catcher = new Catcher(null, async () => {
      return { status: 500, body: "Internal Error Custom" };
    });

    expect(catcher).toBeDefined();
    expect(catcher.code).toBeNull();
    expect(catcher.base).toBe("/");
  });
});

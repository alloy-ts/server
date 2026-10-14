import { describe, expect, test } from "vite-plus/test";
import { Catcher, response } from "../dist/index.js";
import type { RequestInput } from "../dist/index.js";

describe("Catcher NAPI class bindings", () => {
  test("Catcher.new constructs catcher instance with code", () => {
    const catcher = new Catcher(404, (_req: RequestInput) => {
      return response("Custom 404 Not Found", { status: 404 });
    });

    expect(catcher).toBeDefined();
    expect(catcher.code).toBe(404);
    expect(catcher.base).toBe("/");
    expect(catcher.name).toBeNull();

    catcher.name = "Custom404Catcher";
    expect(catcher.name).toBe("Custom404Catcher");
  });

  test("Catcher.mapBase remaps base mount point", () => {
    const catcher = new Catcher(500, (_req: RequestInput) => {
      return response("Server Error", { status: 500 });
    });

    catcher.mapBase("/api");
    expect(catcher.base).toBe("/api");
  });

  test("Catcher executes handler callback on handle call", async () => {
    const catcher = new Catcher(404, (req: RequestInput) => {
      return response(`Handled error on ${req.path}`, { status: 404 });
    });

    const resp = await catcher.handle({
      method: "GET",
      path: "/missing",
      query: "",
      headers: {},
      params: {},
    });

    expect(resp.status).toBe(404);
    expect(Buffer.from(resp.body!).toString("utf-8")).toBe("Handled error on /missing");
  });
});

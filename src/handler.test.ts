import { describe, expect, it } from "vite-plus/test";
import { Handler } from "../dist/index.js";

describe("Handler NAPI Bindings", () => {
  it("creates a Handler with a JS function", () => {
    const handler = new Handler((req: any) => "hello " + req.url);
    expect(handler.isValid()).toBe(true);
  });

  it("handles null/undefined callbacks", () => {
    const handler = new Handler();
    expect(handler.isValid()).toBe(false);
  });
});

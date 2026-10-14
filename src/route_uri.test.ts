import { describe, expect, it } from "vite-plus/test";
import { RouteUri } from "../dist/index.js";

describe("RouteUri NAPI Bindings", () => {
  it("creates a RouteUri with default base '/'", () => {
    const uri = new RouteUri("/", "/foo/<bar>");
    expect(uri.base).toBe("/");
    expect(uri.path).toBe("/foo/<bar>");
    expect(uri.query).toBeNull();
    expect(uri.asStr()).toBe("/foo/<bar>");
  });

  it("handles base mount points", () => {
    const uri = RouteUri.parse("/base", "/foo/<bar>?a=1");
    expect(uri.base).toBe("/base");
    expect(uri.path).toBe("/base/foo/<bar>");
    expect(uri.query).toBe("a=1");
    expect(uri.asStr()).toBe("/base/foo/<bar>?a=1");
  });
});

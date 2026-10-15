import { describe, expect, test } from "vite-plus/test";
import { RouteUri } from "../dist/index.js";

describe("Rocket RouteUri NAPI Bindings", () => {
  test("creates a RouteUri with base and origin", () => {
    const routeUri = new RouteUri("/base", "/foo/<bar>?a=1");

    expect(routeUri.base).toBe("/base");
    expect(routeUri.unmountedOrigin).toBe("/foo/<bar>?a=1");
    expect(routeUri.origin).toBe("/base/foo/<bar>?a=1");
    expect(routeUri.path()).toBe("/base/foo/<bar>");
    expect(routeUri.query()).toBe("a=1");
    expect(routeUri.asStr()).toBe("/base/foo/<bar>?a=1");
    expect(routeUri.isNormalized()).toBe(true);
  });

  test("handles route URIs without query parameter", () => {
    const routeUri = new RouteUri("/", "/users/<id>");

    expect(routeUri.base).toBe("/");
    expect(routeUri.path()).toBe("/users/<id>");
    expect(routeUri.query()).toBeNull();
    expect(routeUri.isNormalized()).toBe(true);
  });

  test("throws error on invalid route URIs", () => {
    expect(() => new RouteUri("invalid base", "invalid uri")).toThrow();
  });
});

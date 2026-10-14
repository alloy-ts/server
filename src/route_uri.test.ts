import { describe, expect, test } from "vite-plus/test";
import { RouteUri } from "../dist/index.js";

describe("Rocket RouteUri NAPI Bindings", () => {
  test("creates RouteUri with base and unmounted origin", () => {
    const routeUri = new RouteUri("/", "/foo/<bar>?a=1");
    expect(routeUri.base).toBe("/");
    expect(routeUri.unmountedOrigin).toBe("/foo/<bar>?a=1");
    expect(routeUri.path()).toBe("/foo/<bar>");
    expect(routeUri.query()).toBe("a=1");
    expect(routeUri.asStr()).toBe("/foo/<bar>?a=1");
  });

  test("handles mounted base path correctly", () => {
    const routeUri = new RouteUri("/base", "/foo/<bar>");
    expect(routeUri.base).toBe("/base");
    expect(routeUri.unmountedOrigin).toBe("/foo/<bar>");
    expect(routeUri.path()).toBe("/base/foo/<bar>");
    expect(routeUri.asStr()).toBe("/base/foo/<bar>");
  });

  test("handles URI without query string", () => {
    const routeUri = new RouteUri("/", "/status");
    expect(routeUri.query()).toBeFalsy();
    expect(routeUri.path()).toBe("/status");
  });
});

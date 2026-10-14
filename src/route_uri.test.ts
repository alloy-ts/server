import { expect, test, describe } from "vite-plus/test";
import { RouteUri } from "../dist/index.js";

describe("Rocket RouteUri NAPI bindings", () => {
  describe("RouteUri constructor and property getters", () => {
    test("creates RouteUri with root base", () => {
      const uri = new RouteUri("/", "/foo/bar?a=1");
      expect(uri).toBeInstanceOf(RouteUri);
      expect(uri.base).toBe("/");
      expect(uri.path).toBe("/foo/bar");
      expect(uri.query).toBe("a=1");
      expect(uri.asStr).toBe("/foo/bar?a=1");
      expect(uri.unmountedOrigin).toBe("/foo/bar?a=1");
    });

    test("creates RouteUri with mounted base path", () => {
      const uri = new RouteUri("/base", "/foo/<bar>");
      expect(uri.base).toBe("/base");
      expect(uri.path).toBe("/base/foo/<bar>");
      expect(uri.asStr).toBe("/base/foo/<bar>");
      expect(uri.unmountedOrigin).toBe("/foo/<bar>");
      expect(uri.query).toBeNull();
    });
  });

  describe("methods", () => {
    test("isNormalized returns true for normalized route URIs", () => {
      const uri = new RouteUri("/", "/a/b/c");
      expect(uri.isNormalized()).toBe(true);
    });

    test("toString returns string representation", () => {
      const uri = new RouteUri("/base", "/foo/bar?a=1");
      expect(uri.toString()).toBe("/base/foo/bar?a=1");
    });
  });
});

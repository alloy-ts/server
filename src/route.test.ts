import { describe, expect, test } from "vite-plus/test";
import { Route } from "../dist/index.js";

describe("Rocket Route NAPI Bindings", () => {
  describe("Route.new() and default ranks", () => {
    test("creates a route with computed default rank", () => {
      const index = new Route("GET", "/");
      expect(index.rank).toBe(-9);
      expect(index.method).toBe("GET");
      expect(index.uri).toBe("/");
    });

    test("computes ranks according to Rocket default ranking table", () => {
      // static path, static query -> -12
      expect(Route.ranked(null, "GET", "/?foo").rank).toBe(-12);
      expect(Route.ranked(null, "GET", "/foo/bar?a=b&bob").rank).toBe(-12);

      // static path, partial query -> -11
      expect(Route.ranked(null, "GET", "/?a&<zoo..>").rank).toBe(-11);

      // static path, wild query -> -10
      expect(Route.ranked(null, "GET", "/?<zoo..>").rank).toBe(-10);

      // static path, no query -> -9
      expect(Route.ranked(null, "GET", "/").rank).toBe(-9);

      // partial path, static query -> -8
      expect(Route.ranked(null, "GET", "/a/<b>?foo").rank).toBe(-8);

      // partial path, no query -> -5
      expect(Route.ranked(null, "GET", "/a/<b>").rank).toBe(-5);

      // wild path, no query -> -1
      expect(Route.ranked(null, "GET", "/<b>/<c>").rank).toBe(-1);
    });
  });

  describe("Route.ranked() with explicit rank", () => {
    test("preserves explicit rank when provided", () => {
      const foo = Route.ranked(1, "POST", "/foo?bar");
      expect(foo.rank).toBe(1);
      expect(foo.method).toBe("POST");
      expect(foo.uri).toBe("/foo?bar");
    });
  });

  describe("mapBase()", () => {
    test("prepends base prefix to route URI", () => {
      const route = new Route("GET", "/foo/bar");
      expect(route.uri).toBe("/foo/bar");

      const mapped = route.mapBase("/boo");
      expect(mapped.uri).toBe("/boo/foo/bar");
    });
  });

  describe("Getters and Setters", () => {
    test("allows setting and reading route properties", () => {
      const route = new Route("GET", "/");
      route.name = "index_route";
      route.format = "json";
      route.rank = 5;

      expect(route.name).toBe("index_route");
      expect(route.format).toBe("json");
      expect(route.rank).toBe(5);
    });
  });
});

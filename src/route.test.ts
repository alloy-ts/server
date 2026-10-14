import { expect, test, describe } from "vite-plus/test";
import { Route } from "../dist/index.js";

describe("Rocket Route NAPI bindings", () => {
  describe("Route constructor and default ranking table", () => {
    test("creates Route instance with default computed ranks", () => {
      const index = new Route("GET", "/");
      expect(index).toBeInstanceOf(Route);
      expect(index.method).toBe("GET");
      expect(index.uri).toBe("/");
      expect(index.rank).toBe(-9);
    });

    test("computes default ranks accurately per Rocket specification", () => {
      // static path, static query => -12
      expect(new Route("GET", "/?foo").rank).toBe(-12);
      expect(new Route("GET", "/foo/bar?a=b&bob").rank).toBe(-12);

      // static path, partial query => -11
      expect(new Route("GET", "/?a&<zoo..>").rank).toBe(-11);

      // static path, wild query => -10
      expect(new Route("GET", "/?<zoo..>").rank).toBe(-10);

      // static path, no query => -9
      expect(new Route("GET", "/").rank).toBe(-9);
      expect(new Route("GET", "/foo/bar").rank).toBe(-9);

      // partial path, static query => -8
      expect(new Route("GET", "/a/<b>?foo").rank).toBe(-8);

      // partial path, partial query => -7
      expect(new Route("GET", "/a/<b>?<b>&c").rank).toBe(-7);

      // partial path, wild query => -6
      expect(new Route("GET", "/a/<b>?<c..>").rank).toBe(-6);

      // partial path, no query => -5
      expect(new Route("GET", "/a/<b>").rank).toBe(-5);

      // wild path, static query => -4
      expect(new Route("GET", "/<b>/<c>?foo&bar").rank).toBe(-4);

      // wild path, partial query => -3
      expect(new Route("GET", "/<b>/<c>?<foo>&bar").rank).toBe(-3);

      // wild path, wild query => -2
      expect(new Route("GET", "/<b>/<c>?<foo>").rank).toBe(-2);

      // wild path, no query => -1
      expect(new Route("GET", "/<b>/<c>").rank).toBe(-1);
    });
  });

  describe("Route.ranked factory method", () => {
    test("creates route with explicit rank", () => {
      const foo = Route.ranked(1, "POST", "/foo?bar");
      expect(foo.rank).toBe(1);
      expect(foo.method).toBe("POST");
      expect(foo.uri).toBe("/foo?bar");
    });

    test("uses computed default rank when rank parameter is null or undefined", () => {
      const foo = Route.ranked(null, "POST", "/foo?bar");
      expect(foo.rank).toBe(-12);
      expect(foo.method).toBe("POST");
      expect(foo.uri).toBe("/foo?bar");
    });
  });

  describe("mapBase and toString methods", () => {
    test("mapBase maps the base URI of a route", () => {
      const index = new Route("GET", "/foo/bar");
      expect(index.uri).toBe("/foo/bar");

      const mapped = index.mapBase("/boo");
      expect(mapped.uri).toBe("/boo/foo/bar");
    });

    test("toString returns formatted string representation", () => {
      const route = new Route("GET", "/users/<id>");
      const str = route.toString();
      expect(str).toContain("GET");
      expect(str).toContain("/users/<id>");
    });
  });

  describe("error validation", () => {
    test("throws error when invalid HTTP method is supplied", () => {
      expect(() => new Route("INVALID_METHOD", "/")).toThrow(/invalid HTTP method/);
    });
  });
});

import { describe, expect, test } from "vite-plus/test";
import { Route } from "./index.ts";

describe("Rocket Route NAPI Bindings", () => {
  test("creates a new route with default computed rank", () => {
    const route = new Route("GET", "/");
    expect(route.method).toBe("GET");
    expect(route.uri).toBe("/");
    expect(route.rank).toBe(-9);
    expect(route.name).toBeNull();
    expect(route.format).toBeNull();
  });

  test("Route.newRoute alias creates route with computed default rank", () => {
    const route = Route.newRoute("POST", "/foo/bar");
    expect(route.method).toBe("POST");
    expect(route.uri).toBe("/foo/bar");
    expect(route.rank).toBe(-9);
  });

  test("Route.ranked creates route with explicit rank", () => {
    const route = Route.ranked(1, "POST", "/foo?bar");
    expect(route.method).toBe("POST");
    expect(route.uri).toBe("/foo?bar");
    expect(route.rank).toBe(1);
  });

  test("Route.ranked with null rank uses computed default rank", () => {
    const route = Route.ranked(null, "POST", "/foo?bar");
    expect(route.method).toBe("POST");
    expect(route.uri).toBe("/foo?bar");
    expect(route.rank).toBe(-12);
  });

  describe("Default Ranking Table Calculation", () => {
    const ranks: Array<[string, number]> = [
      ["/?foo", -12], // static path, static query
      ["/foo/bar?a=b&bob", -12], // static path, static query
      ["/?a&<zoo..>", -11], // static path, partial query
      ["/?<zoo..>", -10], // static path, wild query
      ["/", -9], // static path, no query
      ["/foo/bar", -9], // static path, no query
      ["/a/<b>?foo", -8], // partial path, static query
      ["/a/<b>?<b>&c", -7], // partial path, partial query
      ["/a/<b..>?<c..>", -6], // partial path, wild query
      ["/a/<b>", -5], // partial path, no query
      ["/<b>/<c>?foo&bar", -4], // wild path, static query
      ["/<b>/<c>?<foo>&bar", -3], // wild path, partial query
      ["/<b>/<c>?<foo>", -2], // wild path, wild query
      ["/<b>/<c>", -1], // wild path, no query
    ];

    for (const [uriPattern, expectedRank] of ranks) {
      test(`URI "${uriPattern}" computes rank ${expectedRank}`, () => {
        const route = new Route("GET", uriPattern);
        expect(route.rank).toBe(expectedRank);
      });
    }
  });

  test("Route.create sets explicit name, rank, and format", () => {
    const route = Route.create("get_user", "GET", "/user", 2, "json");
    expect(route.name).toBe("get_user");
    expect(route.method).toBe("GET");
    expect(route.uri).toBe("/user");
    expect(route.rank).toBe(2);
    expect(route.format).toBe("json");
  });

  test("getters and setters mutate properties and validate inputs", () => {
    const route = new Route("GET", "/");

    route.name = "my_route";
    expect(route.name).toBe("my_route");

    route.name = null;
    expect(route.name).toBeNull();

    route.method = "post";
    expect(route.method).toBe("POST");

    route.rank = 5;
    expect(route.rank).toBe(5);

    route.format = "json";
    expect(route.format).toBe("json");

    route.format = null;
    expect(route.format).toBeNull();

    expect(() => {
      route.method = "INVALID_METHOD";
    }).toThrow(/invalid HTTP method/);

    expect(() => {
      route.format = "invalid/format/type";
    }).toThrow(/invalid media type/);
  });
});

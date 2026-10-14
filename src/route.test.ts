import { describe, expect, it } from "vite-plus/test";
import { Route } from "../dist/index.js";

describe("Route NAPI Bindings", () => {
  it("creates a Route with default computed rank", () => {
    const index = new Route("GET", "/");
    expect(index.rank).toBe(-9);
    expect(index.method).toBe("GET");
    expect(index.uri).toBe("/");
  });

  it("creates a ranked Route with custom rank", () => {
    const route = Route.ranked("POST", "/foo?bar", 1);
    expect(route.rank).toBe(1);
    expect(route.method).toBe("POST");
    expect(route.uri).toBe("/foo?bar");
  });

  it("calculates Rocket default ranks according to URI patterns", () => {
    // static path, static query -> -12
    expect(Route.defaultRank("/?foo")).toBe(-12);
    expect(Route.defaultRank("/foo/bar?a=b&bob")).toBe(-12);

    // static path, partial query -> -11
    expect(Route.defaultRank("/?a&<zoo..>")).toBe(-11);

    // static path, wild query -> -10
    expect(Route.defaultRank("/?<zoo..>")).toBe(-10);

    // static path, no query -> -9
    expect(Route.defaultRank("/")).toBe(-9);
    expect(Route.defaultRank("/foo/bar")).toBe(-9);

    // partial path, static query -> -8
    expect(Route.defaultRank("/a/<b>?foo")).toBe(-8);

    // partial path, partial query -> -7
    expect(Route.defaultRank("/a/<b>?<b>&c")).toBe(-7);

    // partial path, wild query -> -6
    expect(Route.defaultRank("/a/<b>?<c..>")).toBe(-6);

    // partial path, no query -> -5
    expect(Route.defaultRank("/a/<b>")).toBe(-5);

    // wild path, static query -> -4
    expect(Route.defaultRank("/<b>/<c>?foo&bar")).toBe(-4);

    // wild path, partial query -> -3
    expect(Route.defaultRank("/<b>/<c>?<foo>&bar")).toBe(-3);

    // wild path, wild query -> -2
    expect(Route.defaultRank("/<b>/<c>?<foo>")).toBe(-2);

    // wild path, no query -> -1
    expect(Route.defaultRank("/<b>/<c>")).toBe(-1);
    expect(Route.defaultRank("/<b..>")).toBe(-1);
  });
});

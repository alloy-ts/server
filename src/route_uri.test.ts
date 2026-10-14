import { describe, expect, test } from "vite-plus/test";
import { RouteUri } from "../dist/index.js";

describe("RouteUri NAPI class bindings", () => {
  test("RouteUri.new creates route URI with base and origin", () => {
    const uri = new RouteUri("/", "/foo/bar?a=1");
    expect(uri.base).toBe("/");
    expect(uri.unmountedOrigin).toBe("/foo/bar?a=1");
    expect(uri.origin).toBe("/foo/bar?a=1");
    expect(uri.path).toBe("/foo/bar");
    expect(uri.query).toBe("a=1");
    expect(uri.asStr).toBe("/foo/bar?a=1");
  });

  test("RouteUri.mapBase remaps base mount point", () => {
    const uri = new RouteUri("/", "/foo/bar?a=1");
    const remapped = uri.mapBase("/boo");
    expect(remapped.base).toBe("/boo");
    expect(remapped.unmountedOrigin).toBe("/foo/bar?a=1");
    expect(remapped.origin).toBe("/boo/foo/bar?a=1");
    expect(remapped.path).toBe("/boo/foo/bar");
    expect(remapped.query).toBe("a=1");
  });

  test("RouteUri handles queries and default ranks", () => {
    const staticUri = new RouteUri("/", "/users/list");
    expect(staticUri.query).toBeNull();
    expect(staticUri.defaultRank).toBe(-9);

    const partialUri = new RouteUri("/", "/users/:id");
    expect(partialUri.defaultRank).toBe(-5);

    const wildUri = new RouteUri("/", "/:id");
    expect(wildUri.defaultRank).toBe(-1);
  });
});

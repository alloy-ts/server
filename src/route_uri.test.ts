import { describe, expect, test } from "vite-plus/test";
import { Route } from "../dist/index.js";

describe("RouteUri getters", () => {
  test("inspects route URI base, path, query, and asStr", () => {
    const route = new Route("GET", "/foo/bar?a=1");
    const uri = route.mapBase("/boo").uri;

    expect(uri).toBe("/boo/foo/bar?a=1");
  });
});

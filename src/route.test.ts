import { describe, expect, it } from "vite-plus/test";
import { Route } from "../dist/index.js";

describe("Route NAPI bindings", () => {
  it("instantiates Route with method and uri", () => {
    const route = new Route("GET", "/users/:id");
    expect(route).toBeDefined();
    expect(route.method).toBe("GET");
    expect(route.uri).toBe("/users/:id");
    expect(route.name).toBeNull();
    expect(route.format).toBeNull();
    expect(typeof route.rank).toBe("number");
  });

  it("instantiates ranked Route with explicit rank", () => {
    const route = Route.ranked(-10, "POST", "/api/posts");
    expect(route).toBeDefined();
    expect(route.method).toBe("POST");
    expect(route.uri).toBe("/api/posts");
    expect(route.rank).toBe(-10);
  });
});

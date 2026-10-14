import { describe, expect, test } from "vite-plus/test";
import { Route } from "../dist/index.js";

describe("Route NAPI class", () => {
  test("creates route with default computed rank", () => {
    const route = new Route("GET", "/users/<id>");
    expect(route.method).toBe("GET");
    expect(route.uri).toBe("/users/<id>");
    expect(typeof route.rank).toBe("number");
  });

  test("creates ranked route", () => {
    const route = Route.ranked(2, "POST", "/api/items");
    expect(route.rank).toBe(2);
    expect(route.method).toBe("POST");
    expect(route.uri).toBe("/api/items");
  });

  test("maps base URI using mapBase", () => {
    const route = new Route("GET", "/posts");
    const rebased = route.mapBase("/v1");
    expect(rebased.uri).toBe("/v1/posts");
  });
});

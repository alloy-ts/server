import { describe, expect, it } from "vite-plus/test";
import { response, Route } from "../dist/index.js";

describe("Route NAPI class bindings", () => {
  it("Route.new() creates route with computed default rank", () => {
    const route = new Route("GET", "/", () => response("index"));

    expect(route.method).toBe("GET");
    expect(route.uri).toBe("/");
    expect(route.rank).toBe(-9); // static path, no query
    expect(route.name).toBeNull();
    expect(route.format).toBeNull();
  });

  it("Route.ranked() sets explicit rank", () => {
    const route = Route.ranked(2, "POST", "/route/<path..>?query", () => response("ok"));

    expect(route.rank).toBe(2);
    expect(route.method).toBe("POST");
    expect(route.uri).toBe("/route/<path..>?query");
  });

  it("Route getters and setters inspect and mutate fields", () => {
    const route = new Route("GET", "/users/<id>", () => response("user"));

    route.name = "get_user";
    expect(route.name).toBe("get_user");

    route.rank = 10;
    expect(route.rank).toBe(10);

    route.format = "json";
    expect(route.format).toBe("application/json");

    route.format = null;
    expect(route.format).toBeNull();
  });

  it("Route.mapBase() remaps base URI", () => {
    const route = new Route("GET", "/status", () => response("ok"));
    expect(route.uri).toBe("/status");

    route.mapBase("/api/v1");
    expect(route.uri).toBe("/api/v1/status");
  });
});

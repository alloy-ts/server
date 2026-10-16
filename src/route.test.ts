import test from "node:test";
import assert from "node:assert/strict";
import { Route } from "../index.js";

test("Route initialization and getters", () => {
  const handler = () => {};
  const route = new Route("GET", "/users/<id>", handler);

  assert.equal(route.method, "GET");
  assert.equal(route.uri, "/users/<id>");
  assert.equal(typeof route.rank, "number");
});

test("Route setters and modification", () => {
  const route = new Route("POST", "/items", () => {});

  route.name = "create_item";
  assert.equal(route.name, "create_item");

  route.rank = 10;
  assert.equal(route.rank, 10);

  assert.equal(route.format, null);
});

test("Route mapBase and ranked methods", () => {
  const route = new Route("GET", "/posts", () => {});
  route.name = "get_posts";

  route.mapBase("/api/v1");
  assert.equal(route.uri, "/api/v1/posts");
  assert.equal(route.name, "get_posts");

  const ranked = Route.ranked(-1, "GET", "/posts", () => {});
  assert.equal(ranked.rank, -1);
});

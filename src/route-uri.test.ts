import test from "node:test";
import assert from "node:assert/strict";
import { RouteUri } from "../index.js";

test("RouteUri constructor and getters", () => {
  const routeUri = new RouteUri("/api/v1", "/users/<id>?active=true#section");

  assert.equal(routeUri.base, "/api/v1");
  assert.equal(routeUri.unmountedOrigin, "/users/<id>?active=true#section");
  assert.equal(routeUri.origin, "/api/v1/users/<id>?active=true#section");
  assert.equal(routeUri.path, "/api/v1/users/<id>");
  assert.equal(routeUri.query, "active=true#section");
  assert.equal(routeUri.asStr, "/api/v1/users/<id>?active=true#section");
});

test("RouteUri with root base", () => {
  const routeUri = new RouteUri("/", "/items");

  assert.equal(routeUri.base, "/");
  assert.equal(routeUri.unmountedOrigin, "/items");
  assert.equal(routeUri.origin, "/items");
  assert.equal(routeUri.path, "/items");
  assert.equal(routeUri.query, null);
});

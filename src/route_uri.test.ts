import { expect, test } from "vite-plus/test";
import { RouteUri } from "../index.js";

test("RouteUri base, unmountedOrigin, origin, path, and query", () => {
  const u = new RouteUri("/api", "/users/<id>?active=true");
  expect(u.base).toBe("/api");
  expect(u.unmountedOrigin).toBe("/users/<id>?active=true");
  expect(u.origin).toBe("/api/users/<id>?active=true");
  expect(u.path()).toBe("/api/users/<id>");
  expect(u.query()).toBe("active=true");
  expect(u.asStr()).toBe("/api/users/<id>?active=true");
});

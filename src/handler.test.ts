import { expect, test } from "vite-plus/test";
import { Route, handlerSuccess, handlerError, handlerForward, response } from "../dist/index.js";

test("Route constructor and getters/setters", () => {
  const route = new Route("GET", "/api/items", "getItems", "json");
  expect(route.method).toBe("GET");
  expect(route.uri).toBe("/api/items");
  expect(route.name).toBe("getItems");
  expect(route.format).toBe("json");
  expect(route.rank).toBe(-9);

  route.mapBase("/v1");
  expect(route.uri).toBe("/v1/api/items");

  route.rank = 1;
  expect(route.rank).toBe(1);
});

test("Route ranked constructor", () => {
  const route = Route.ranked(2, "POST", "/users", "createUser", null);
  expect(route.rank).toBe(2);
  expect(route.method).toBe("POST");
  expect(route.uri).toBe("/users");
});

test("Handler outcome helpers", () => {
  const resp = response("ok");
  const success = handlerSuccess(resp);
  expect(success.status).toBe("success");
  expect(success.response?.status).toBe(200);

  const err = handlerError(400);
  expect(err.status).toBe("error");
  expect(err.code).toBe(400);

  const fwd = handlerForward(404);
  expect(fwd.status).toBe("forward");
  expect(fwd.code).toBe(404);
});

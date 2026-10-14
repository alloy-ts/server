import { expect, test } from "vite-plus/test";
import { Catcher, handlerSuccess, handlerError, handlerForward, response } from "../dist/index.js";

test("Catcher class and getters/setters", () => {
  const catcher = new Catcher(404, "not_found");
  expect(catcher.code).toBe(404);
  expect(catcher.name).toBe("not_found");
  expect(catcher.base).toBe("/");

  catcher.code = 500;
  expect(catcher.code).toBe(500);

  catcher.name = "server_error";
  expect(catcher.name).toBe("server_error");
});

test("Catcher mapBase updates base path", () => {
  const catcher = new Catcher(500, "internal_error");
  expect(catcher.base).toBe("/");

  catcher.mapBase("/api");
  expect(catcher.base).toBe("/api");
});

test("Catcher validates status code range", () => {
  expect(() => new Catcher(200 as any, "invalid")).toThrow();
  expect(() => new Catcher(600 as any, "invalid")).toThrow();
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

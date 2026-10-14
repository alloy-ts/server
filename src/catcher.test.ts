import { expect, test } from "vite-plus/test";
import { Alloy, Catcher, Rocket } from "./main.ts";

test("Catcher creates instances with status codes and defaults", () => {
  const notFound = new Catcher(404, () => new Alloy.Response("Not Found"));
  expect(notFound.code).toBe(404);
  expect(notFound.base).toBe("/");

  const defaultCatcher = new Catcher(undefined, () => new Alloy.Response("Default Error"));
  expect(defaultCatcher.code ?? undefined).toBeUndefined();
  expect(defaultCatcher.base).toBe("/");
});

test("Catcher validates HTTP status code range [400, 600)", () => {
  expect(() => new Catcher(200)).toThrow();
  expect(() => new Catcher(600)).toThrow();
});

test("Catcher maps base URI correctly", () => {
  const catcher = new Catcher(500, () => new Alloy.Response("Server Error"));
  expect(catcher.base).toBe("/");

  catcher.mapBase("/bar");
  expect(catcher.base).toBe("/bar");

  catcher.mapBase("/foo");
  expect(catcher.base).toBe("/foo/bar");
});

test("Rocket accepts registered catchers", () => {
  const rocket = new Rocket();
  const handler = () => new Alloy.Response("Custom 404", { status: 404 });
  expect(() => rocket.register("/", 404, handler)).not.toThrow();
});

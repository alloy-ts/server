import { expect, test } from "vite-plus/test";
import { Request } from "../dist/index.js";

test("Request constructor and method/uri getters/setters", () => {
  const req = new Request("GET", "/hello");
  expect(req.method()).toBe("GET");
  expect(req.uri()).toBe("/hello");

  req.setMethod("POST");
  expect(req.method()).toBe("POST");

  req.setUri("/hello/world");
  expect(req.uri()).toBe("/hello/world");
});

test("Request optional IP and header getters", () => {
  const req = new Request("GET", "/");
  expect(req.remote()).toBeNull();
  expect(req.realIp()).toBeNull();
  expect(req.clientIp()).toBeNull();
  expect(req.contentType()).toBeNull();
  expect(req.accept()).toBeNull();
  expect(req.format()).toBeNull();
});

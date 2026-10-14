import { expect, test } from "vite-plus/test";
import {
  type CatcherInfo,
  Rocket,
  createCatcherInfo,
  defaultCatcherResponse,
} from "../dist/index.js";

test("createCatcherInfo constructor and status validation", () => {
  const info404: CatcherInfo = createCatcherInfo(404, "/api", "not_found");
  expect(info404.code).toBe(404);
  expect(info404.base).toBe("/api");
  expect(info404.name).toBe("not_found");

  const defaultInfo: CatcherInfo = createCatcherInfo(undefined);
  expect(defaultInfo.code).toBeUndefined();
  expect(defaultInfo.base).toBe("/");

  expect(() => createCatcherInfo(200)).toThrow(/outside valid error range/);
  expect(() => createCatcherInfo(600)).toThrow(/outside valid error range/);
});

test("defaultCatcherResponse HTML and JSON generation", () => {
  const htmlResp = defaultCatcherResponse(404, false);
  expect(htmlResp.status).toBe(404);
  expect(htmlResp.headers["Content-Type"]).toBe("text/html; charset=utf-8");
  expect(htmlResp.body).toBeDefined();

  const jsonResp = defaultCatcherResponse(500, true);
  expect(jsonResp.status).toBe(500);
  expect(jsonResp.headers["Content-Type"]).toBe("application/json");
  const jsonString = Buffer.from(jsonResp.body!).toString();
  expect(jsonString).toContain("HTTP Error 500");
});

test("Rocket server catcher registration", () => {
  const rocket = new Rocket({ port: 8083 });

  rocket.register("/", 404, () => ({
    status: 404,
    headers: [{ name: "Content-Type", value: "text/plain" }],
    body: Buffer.from("Custom 404 Not Found"),
  }));

  expect(rocket).toBeDefined();
});

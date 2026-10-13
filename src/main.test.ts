import { expect, test } from "vite-plus/test";
import { Service, response } from "../dist/index.js";

test("Service registers a route and exposes launch", () => {
  const app = new Service();
  app.get("/", () => response("Hello, world!"));
  expect(typeof app.launch).toBe("function");
});

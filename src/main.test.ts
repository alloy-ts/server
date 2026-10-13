import { expect, test } from "vite-plus/test";
import { Alloy, del, get, main, post, put, Response, uri } from "./main.ts";

test("main returns Hello, world!", () => {
  expect(main()).toBe("Hello, world!");
});

test("Response constructor and JSON creation", () => {
  const resp = new Response("Hello");
  expect(resp.status).toBe(200);

  const jsonResp = Response.json({ status: "ok" });
  expect(jsonResp.status).toBe(200);
});

test("uri builder helper for codegen", () => {
  expect(uri("/users/:id", { id: "123" })).toBe("/users/123");
  expect(uri("/posts/<slug>", { slug: "hello-world" })).toBe("/posts/hello-world");
});

test("route decorators definition", () => {
  const handler = () => new Response("Hello");
  class Controller {}

  get("/hello")({ constructor: Controller }, "getHello", { value: handler });
  post("/hello")({ constructor: Controller }, "postHello", { value: handler });
  put("/hello")({ constructor: Controller }, "putHello", { value: handler });
  del("/hello")({ constructor: Controller }, "delHello", { value: handler });

  expect(Controller).toBeDefined();
});

test("Alloy serve facade export", () => {
  expect(typeof Alloy.serve).toBe("function");
});

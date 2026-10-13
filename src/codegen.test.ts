import { expect, test } from "vite-plus/test";
import { del, get, post, put, Response, uri } from "./main.ts";

test("codegen decorators and routes", () => {
  const handler = () => new Response("Hello Codegen");
  class Controller {}

  get("/codegen/get")({ constructor: Controller }, "get", { value: handler });
  post("/codegen/post")({ constructor: Controller }, "post", { value: handler });
  put("/codegen/put")({ constructor: Controller }, "put", { value: handler });
  del("/codegen/del")({ constructor: Controller }, "del", { value: handler });

  expect(Controller).toBeDefined();
});

test("codegen uri helper", () => {
  expect(uri("/api/user/:id", { id: 100 })).toBe("/api/user/100");
  expect(uri("/api/item/<id>", { id: "abc" })).toBe("/api/item/abc");
});

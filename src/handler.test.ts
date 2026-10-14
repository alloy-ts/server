import { expect, test } from "vite-plus/test";
import { Handler, response, responseJson } from "../dist/index.js";

test("Handler class instantiates from JS callback", () => {
  const handler = new Handler(async (req) => {
    if (req.path === "/json") {
      return responseJson({ ok: true });
    }
    return response("Hello from custom Handler");
  });

  expect(handler).toBeDefined();
  expect(typeof handler).toBe("object");
});

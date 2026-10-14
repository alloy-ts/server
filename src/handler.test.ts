import { expect, test, describe } from "vite-plus/test";
import { Handler, Route, response, responseJson, type RequestInput } from "../dist/index.js";

describe("Rocket Handler NAPI bindings", () => {
  test("new Handler creates Handler instance", () => {
    const handler = new Handler(async () => response("OK"));
    expect(handler).toBeInstanceOf(Handler);
  });

  test("handler.toRoute creates Route object", () => {
    const handler = new Handler(async (req: RequestInput) => {
      return responseJson({ path: req.path });
    });

    const route = handler.toRoute("GET", "/custom");
    expect(route).toBeInstanceOf(Route);
    expect(route.method).toBe("GET");
    expect(route.uri).toBe("/custom");
    expect(route.rank).toBe(-9);
  });
});

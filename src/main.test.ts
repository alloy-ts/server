import { expect, test } from "vite-plus/test";
import { serve, response, Config, Route } from "../dist/index.js";

test("NAPI exports serve, response, Config, Route", () => {
  expect(typeof serve).toBe("function");
  expect(typeof response).toBe("function");
  expect(typeof Config).toBe("function");
  expect(typeof Route).toBe("function");
});

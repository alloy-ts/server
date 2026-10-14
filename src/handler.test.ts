import { expect, test } from "vite-plus/test";
import { Handler } from "../dist/index.js";

test("Handler constructor creates named handler", () => {
  const handler = new Handler("myHandler");
  expect(handler.name).toBe("myHandler");
});

test("Handler constructor with empty name", () => {
  const handler = new Handler();
  expect(handler.name).toBeNull();
});

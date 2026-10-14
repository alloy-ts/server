import { expect, test } from "vite-plus/test";
import { RocketHandler } from "./main.ts";

test("RocketHandler constructor creates named handler", () => {
  const handler = new RocketHandler("custom_handler");
  expect(handler.name).toBe("custom_handler");
});

test("RocketHandler constructor allows optional name", () => {
  const handler = new RocketHandler();
  expect(handler.name).toBeNull();
});

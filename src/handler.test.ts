import { expect, test } from "vite-plus/test";
import { Handler } from "../dist/index.js";

test("Handler constructor and getters", () => {
  const handler = new Handler("my_handler");
  expect(handler.name).toBe("my_handler");
});

test("Handler outcomes", () => {
  const successOutcome = Handler.success({ status: 200, headers: [] });
  expect(successOutcome.status).toBe(200);
  expect(successOutcome.isForward).toBe(false);

  const errorOutcome = Handler.error(500);
  expect(errorOutcome.status).toBe(500);
  expect(errorOutcome.isForward).toBe(false);

  const forwardOutcome = Handler.forward(404);
  expect(forwardOutcome.status).toBe(404);
  expect(forwardOutcome.isForward).toBe(true);
});

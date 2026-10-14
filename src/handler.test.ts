import { expect, test } from "vite-plus/test";
import { Handler, Outcome, type OutcomeStatus } from "./main.ts";

test("Outcome static constructors and fields", () => {
  const succ = Outcome.success(Buffer.from("hello"));
  expect(succ.status).toBe(0 as OutcomeStatus.Success);
  expect(succ.code).toBe(200);
  expect(succ.body ? Buffer.from(succ.body).toString("utf-8") : "").toBe("hello");

  const err = Outcome.error(404);
  expect(err.status).toBe(1 as OutcomeStatus.Error);
  expect(err.code).toBe(404);
  expect(err.body).toBeNull();

  const fwd = Outcome.forward(400);
  expect(fwd.status).toBe(2 as OutcomeStatus.Forward);
  expect(fwd.code).toBe(400);
  expect(fwd.body).toBeNull();
});

test("Handler class instantiation and getter", () => {
  const handler1 = new Handler();
  expect(handler1.handlerType).toBe("custom");

  const handler2 = new Handler("my-custom-plugin");
  expect(handler2.handlerType).toBe("my-custom-plugin");
});

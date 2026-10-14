import { expect, test } from "vite-plus/test";
import {
  createHandlerOutcomeSuccess,
  createHandlerOutcomeError,
  createHandlerOutcomeForward,
} from "./main.ts";

test("createHandlerOutcomeSuccess constructs success outcome", () => {
  const success = createHandlerOutcomeSuccess("hello", 200);
  expect(success.status).toBe(200);
  expect(success.body).toBe("hello");
  expect(success.forwarded).toBe(false);
});

test("createHandlerOutcomeError constructs error outcome", () => {
  const error = createHandlerOutcomeError(400);
  expect(error.status).toBe(400);
  expect(error.body).toBeUndefined();
  expect(error.forwarded).toBe(false);
});

test("createHandlerOutcomeForward constructs forward outcome", () => {
  const forward = createHandlerOutcomeForward();
  expect(forward.status).toBe(404);
  expect(forward.forwarded).toBe(true);
});

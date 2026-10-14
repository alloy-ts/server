import { expect, test } from "vite-plus/test";
import {
  handlerOutcomeError,
  handlerOutcomeForward,
  handlerOutcomeSuccess,
  jsonResponse,
  textResponse,
} from "../dist/index.js";

test("textResponse creates a text/plain response payload", () => {
  const resp = textResponse("simple", 200);
  expect(resp.status).toBe(200);
  expect(resp.headers["Content-Type"]).toBe("text/plain; charset=utf-8");
  expect(resp.body).toBeDefined();
});

test("jsonResponse creates an application/json response payload", () => {
  const resp = jsonResponse({ kind: "complex" }, 201);
  expect(resp.status).toBe(201);
  expect(resp.headers["Content-Type"]).toBe("application/json");
  expect(resp.body).toBeDefined();
});

test("handlerOutcomeSuccess creates a success outcome", () => {
  const resp = textResponse("hello");
  const outcome = handlerOutcomeSuccess(resp);

  expect(outcome.kind).toBe("success");
  expect(outcome.status).toBe(200);
  expect(outcome.response?.status).toBe(200);
});

test("handlerOutcomeError creates an error outcome", () => {
  const outcome = handlerOutcomeError(404);

  expect(outcome.kind).toBe("error");
  expect(outcome.status).toBe(404);
  expect(outcome.response).toBeUndefined();
});

test("handlerOutcomeForward creates a forward outcome", () => {
  const outcome = handlerOutcomeForward();

  expect(outcome.kind).toBe("forward");
  expect(outcome.status).toBeUndefined();
  expect(outcome.response).toBeUndefined();
});

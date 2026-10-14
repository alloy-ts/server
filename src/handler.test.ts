import { expect, test } from "vite-plus/test";
import { response, reqText, reqJson } from "../dist/index.js";

test("HandlerOutcome success outcome structure", () => {
  const res = response("Success Content", { status: 200 });
  const outcome = {
    kind: "success",
    response: res,
  };

  expect(outcome.kind).toBe("success");
  expect(outcome.response?.status).toBe(200);
});

test("HandlerOutcome error outcome structure", () => {
  const outcome = {
    kind: "error",
    status: 404,
  };

  expect(outcome.kind).toBe("error");
  expect(outcome.status).toBe(404);
});

test("HandlerOutcome forward outcome structure", () => {
  const outcome = {
    kind: "forward",
    status: 400,
  };

  expect(outcome.kind).toBe("forward");
  expect(outcome.status).toBe(400);
});

test("reqText converts byte array to UTF-8 string", () => {
  const encoder = new TextEncoder();
  const bytes = Array.from(encoder.encode("Hello, Rocket!"));
  const text = reqText(bytes);
  expect(text).toBe("Hello, Rocket!");
});

test("reqText handles null/empty body", () => {
  expect(reqText(null)).toBe("");
  expect(reqText([])).toBe("");
});

test("reqJson converts JSON byte array to JavaScript object", () => {
  const encoder = new TextEncoder();
  const jsonStr = JSON.stringify({ key: "value", count: 42 });
  const bytes = Array.from(encoder.encode(jsonStr));
  const parsed = reqJson(bytes);
  expect(parsed).toEqual({ key: "value", count: 42 });
});

test("reqJson handles invalid or null body", () => {
  expect(reqJson(null)).toBeNull();
  expect(reqJson([])).toBeNull();

  const encoder = new TextEncoder();
  const invalidBytes = Array.from(encoder.encode("not valid json"));
  expect(reqJson(invalidBytes)).toBeNull();
});

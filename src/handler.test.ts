import { describe, expect, test } from "vite-plus/test";
import {
  handlerOutcomeError,
  handlerOutcomeForward,
  handlerOutcomeSuccess,
} from "../dist/index.js";

describe("Rocket Handler Outcome NAPI Bindings", () => {
  test("handlerOutcomeSuccess creates a Success outcome with response", () => {
    const outcome = handlerOutcomeSuccess(
      200,
      [{ name: "Content-Type", value: "text/plain" }],
      Array.from(Buffer.from("OK")),
    );
    expect(outcome.kind).toBe("Success");
    expect(outcome.status).toBe(200);
    expect(outcome.response).toBeDefined();
    expect(outcome.response?.status).toBe(200);
    expect(outcome.response?.headers?.[0]?.name).toBe("Content-Type");
  });

  test("handlerOutcomeError creates an Error outcome with status", () => {
    const outcome = handlerOutcomeError(400);
    expect(outcome.kind).toBe("Error");
    expect(outcome.status).toBe(400);
    expect(outcome.response).toBeUndefined();
  });

  test("handlerOutcomeForward creates a Forward outcome", () => {
    const outcome = handlerOutcomeForward(404);
    expect(outcome.kind).toBe("Forward");
    expect(outcome.status).toBe(404);
    expect(outcome.response).toBeUndefined();
  });
});

import { describe, expect, test } from "vite-plus/test";
import { response, routeOutcomeError, routeOutcomeForward, routeOutcomeSuccess } from "./index.ts";
import type { HandlerOutcomeKind } from "./index.ts";

describe("Rocket Handler NAPI Bindings", () => {
  test("routeOutcomeSuccess creates a Success outcome with response", () => {
    const res = response("Hello World");
    const outcome = routeOutcomeSuccess(res);

    const kind: HandlerOutcomeKind = outcome.kind;
    expect(kind).toBe(0);
    expect(outcome.response).toEqual(res);
    expect(outcome.status).toBe(200);
  });

  test("routeOutcomeError creates an Error outcome with status code", () => {
    const outcome = routeOutcomeError(500);

    const kind: HandlerOutcomeKind = outcome.kind;
    expect(kind).toBe(1);
    expect(outcome.response).toBeUndefined();
    expect(outcome.status).toBe(500);
  });

  test("routeOutcomeForward creates a Forward outcome with default or custom status", () => {
    const outcomeDefault = routeOutcomeForward();
    const kindDefault: HandlerOutcomeKind = outcomeDefault.kind;
    expect(kindDefault).toBe(2);
    expect(outcomeDefault.response).toBeUndefined();
    expect(outcomeDefault.status).toBe(404);

    const outcomeCustom = routeOutcomeForward(400);
    const kindCustom: HandlerOutcomeKind = outcomeCustom.kind;
    expect(kindCustom).toBe(2);
    expect(outcomeCustom.status).toBe(400);
  });
});

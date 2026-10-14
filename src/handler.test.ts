import { describe, expect, test } from "vite-plus/test";
import {
  Handler,
  createHandlerOutcomeSuccess,
  createHandlerOutcomeError,
  createHandlerOutcomeForward,
} from "../dist/index.js";

describe("Handler NAPI class & outcome constructors", () => {
  test("Handler constructor creates Handler instance with name and executes handle", async () => {
    const handler = new Handler("CustomHandler", async (req) => {
      return createHandlerOutcomeSuccess(200, `Hello from ${req.uri}`, [["X-Test", "1"]]);
    });

    expect(handler).toBeDefined();
    expect(handler.name).toBe("CustomHandler");

    const result = await handler.handle({
      method: "GET",
      uri: "/test",
      headers: [],
    });

    expect(result.kind).toBe(0);
    expect(result.status).toBe(200);
    expect(result.response?.body).toBeDefined();
    expect(Buffer.from(result.response!.body!).toString("utf-8")).toBe("Hello from /test");
  });

  test("createHandlerOutcomeSuccess constructs Success outcome", () => {
    const outcome = createHandlerOutcomeSuccess(200, "OK", [["Content-Type", "text/plain"]]);
    expect(outcome.kind).toBe(0); // HandlerOutcomeKind.Success
    expect(outcome.status).toBe(200);
    expect(outcome.response).toBeDefined();
    expect(outcome.response?.status).toBe(200);
    expect(outcome.response?.body).toBeDefined();
    expect(Buffer.from(outcome.response!.body!).toString("utf-8")).toBe("OK");
  });

  test("createHandlerOutcomeError constructs Error outcome", () => {
    const outcome = createHandlerOutcomeError(404);
    expect(outcome.kind).toBe(1); // HandlerOutcomeKind.Error
    expect(outcome.status).toBe(404);
    expect(outcome.response).toBeUndefined();
  });

  test("createHandlerOutcomeForward constructs Forward outcome", () => {
    const outcome = createHandlerOutcomeForward(302);
    expect(outcome.kind).toBe(2); // HandlerOutcomeKind.Forward
    expect(outcome.status).toBe(302);
    expect(outcome.response).toBeUndefined();
  });
});

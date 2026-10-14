import { describe, expect, test } from "vite-plus/test";
import { HandlerBridge } from "../dist/index.js";

describe("HandlerBridge static outcome helpers", () => {
  test("creates success outcome payload", () => {
    const outcome = HandlerBridge.success({
      status: 200,
      headers: [{ name: "Content-Type", value: "text/plain" }],
      body: Buffer.from("hello"),
    });

    expect(outcome.action).toBe("success");
    expect(outcome.response?.status).toBe(200);
  });

  test("creates error outcome payload", () => {
    const outcome = HandlerBridge.error(400);

    expect(outcome.action).toBe("error");
    expect(outcome.status).toBe(400);
  });

  test("creates forward outcome payload", () => {
    const outcome = HandlerBridge.forward(404);

    expect(outcome.action).toBe("forward");
    expect(outcome.status).toBe(404);
  });
});

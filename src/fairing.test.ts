import { describe, expect, test } from "vite-plus/test";
import { FairingBridge, type RequestInput } from "../dist/index.js";

describe("FairingBridge NAPI class", () => {
  test("creates onRequest ad-hoc fairing", () => {
    const fairing = FairingBridge.onRequest("Request Logger", (_req: RequestInput) => {
      // noop
    });

    expect(fairing.name).toBe("Request Logger");
  });
});

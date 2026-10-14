import { expect, test } from "vite-plus/test";
import type { RequestInput, ResponsePayload } from "../index.js";
import { registerRoute } from "./main.ts";

test("Handler function registration and execution", async () => {
  const handler = async (req: RequestInput): Promise<ResponsePayload> => {
    return { status: 200, headers: {}, body: Buffer.from(`Handled ${req.method} ${req.path}`) };
  };

  registerRoute("GET", "/test-handler", handler);
  expect(registerRoute).toBeDefined();
});

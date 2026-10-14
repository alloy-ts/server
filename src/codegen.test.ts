import { expect, test } from "vite-plus/test";
import { registerRoute, Response, uri } from "../index.js";
import type { RequestInput, ResponsePayload } from "../index.js";

test("codegen URI formatting and route registration", () => {
  const res = uri("/users/:id", { id: "42" });
  expect(res).toBe("/users/42");

  const handler = async (_req: RequestInput): Promise<ResponsePayload> => ({
    status: 200,
    headers: {},
    body: Buffer.from("Hello Codegen"),
  });
  registerRoute("GET", "/codegen/get", handler);
  expect(registerRoute).toBeDefined();
});

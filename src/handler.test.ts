import { expect, test } from "vite-plus/test";
import { Handler } from "./main.ts";

test("Handler outcome static helpers construct valid HandlerOutcome objects", () => {
  const successOutcome = Handler.success({
    status: 200,
    headers: [{ name: "Content-Type", value: "text/plain" }],
  });
  expect(successOutcome.kind).toBe("success");
  expect(successOutcome.response?.status).toBe(200);

  const errorOutcome = Handler.error(403);
  expect(errorOutcome.kind).toBe("error");
  expect(errorOutcome.status).toBe(403);

  const forwardOutcome = Handler.forward();
  expect(forwardOutcome.kind).toBe("forward");
});

test("Handler instance can be constructed with JS callback", () => {
  const customHandler = new Handler((_req: { method: string }) => {
    return Handler.success({ status: 200 });
  });

  expect(customHandler).toBeDefined();
});

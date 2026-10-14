import { expect, test } from "vite-plus/test";
import { CustomHandler } from "./main.ts";

test("CustomHandler instantiates cleanly with JS callback", () => {
  const handler = new CustomHandler(async (_req: any) => {
    return {
      status: "success",
      response: {
        status: 200,
        headers: { "X-Custom": "1" },
        body: Array.from(Buffer.from("custom handler response")),
      },
      errorCode: null,
    };
  });

  expect(handler).toBeDefined();
});

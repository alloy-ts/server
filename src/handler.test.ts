import { expect, test } from "vite-plus/test";
import { Handler, outcomeSuccess, outcomeError, outcomeForward, response } from "../dist/index.js";

test("Outcome factory methods create valid outcomes", () => {
  const successOutcome = outcomeSuccess(response("OK"));
  expect(successOutcome.status).toBe("success");
  expect(successOutcome.response?.status).toBe(200);

  const errorOutcome = outcomeError(404);
  expect(errorOutcome.status).toBe("error");
  expect(errorOutcome.code).toBe(404);

  const forwardOutcome = outcomeForward(null);
  expect(forwardOutcome.status).toBe("forward");
});

test("Handler constructor wraps JS handler function", () => {
  const customHandler = new Handler(async (req: any) => {
    if (req.path === "/simple") {
      return outcomeSuccess(response("simple"));
    }
    return outcomeError(400);
  });

  expect(customHandler).toBeDefined();
});

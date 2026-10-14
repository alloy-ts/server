import { expect, test } from "vite-plus/test";
import {
  createSuccessOutcome,
  createErrorOutcome,
  createForwardOutcome,
  response,
} from "./main.ts";

test("HandlerOutcome success, error, and forward helpers", () => {
  const resp = response("Custom Handler Response");
  const successOutcome = createSuccessOutcome(resp);
  expect(successOutcome.kind).toBe("success");
  expect(successOutcome.response?.status).toBe(200);

  const errorOutcome = createErrorOutcome(500);
  expect(errorOutcome.kind).toBe("error");
  expect(errorOutcome.status).toBe(500);

  const forwardOutcome = createForwardOutcome(404);
  expect(forwardOutcome.kind).toBe("forward");
  expect(forwardOutcome.status).toBe(404);
});

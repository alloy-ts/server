import { expect, test } from "vite-plus/test";
import { outcomeSuccess, outcomeError, outcomeForward } from "../dist/index.js";

test("HandlerOutcome helpers produce outcome objects", () => {
  const success = outcomeSuccess("Success payload");
  expect(success.kind).toBe("success");
  expect(success.response?.status).toBe(200);

  const error = outcomeError(403);
  expect(error.kind).toBe("error");
  expect(error.status).toBe(403);

  const forward = outcomeForward(404);
  expect(forward.kind).toBe("forward");
  expect(forward.status).toBe(404);
});

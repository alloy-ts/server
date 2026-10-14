import { expect, test } from "vite-plus/test";
import { outcomeSuccess, outcomeError, outcomeForward, response } from "../dist/index.js";

test("Handler outcome constructor helpers", () => {
  const resp = response("ok");
  const success = outcomeSuccess(resp);
  expect(success.kind).toBe("success");
  expect(success.response?.status).toBe(200);

  const error = outcomeError(400);
  expect(error.kind).toBe("error");
  expect(error.status).toBe(400);

  const forward = outcomeForward(404);
  expect(forward.kind).toBe("forward");
  expect(forward.status).toBe(404);
});

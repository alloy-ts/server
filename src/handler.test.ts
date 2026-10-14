import { expect, test } from "vite-plus/test";
import {
  Handler,
  outcomeError,
  outcomeForward,
  outcomeFrom,
  response,
} from "../dist/index.js";

test("outcome helpers build valid Outcome objects", () => {
  const resp = response("ok");
  const successOutcome = outcomeFrom(resp);
  expect(successOutcome.kind).toBe("success");
  expect(successOutcome.status).toBe(200);

  const errorOutcome = outcomeError(500);
  expect(errorOutcome.kind).toBe("error");
  expect(errorOutcome.status).toBe(500);

  const forwardOutcome = outcomeForward(404);
  expect(forwardOutcome.kind).toBe("forward");
  expect(forwardOutcome.status).toBe(404);
});

test("Handler constructor instantiates custom Rocket handler", () => {
  const customHandler = new Handler(async () => {
    return response("custom handler result");
  });

  expect(customHandler).toBeDefined();
  expect(typeof customHandler).toBe("object");
});

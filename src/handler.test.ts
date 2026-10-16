import test from "node:test";
import assert from "node:assert/strict";
import {
  Handler,
  outcomeSuccess,
  outcomeError,
  outcomeForward,
} from "../index.js";

test("Handler creation", () => {
  const handler = new Handler(() => {
    return outcomeSuccess({
      status: 200,
      headers: [],
      body: Buffer.from("Hello"),
    });
  });

  assert.ok(handler);
});

test("Outcome helpers", () => {
  const success = outcomeSuccess({
    status: 200,
    body: Buffer.from("OK"),
  });
  assert.equal(success.status, "success");
  assert.equal(success.response?.status, 200);

  const error = outcomeError(404);
  assert.equal(error.status, "error");
  assert.equal(error.errorCode, 404);

  const forward = outcomeForward(500);
  assert.equal(forward.status, "forward");
  assert.equal(forward.errorCode, 500);
});

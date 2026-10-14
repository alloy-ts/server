import { expect, test } from "vite-plus/test";
import { OutcomeHelper, OutcomeKind, response } from "../dist/index.js";

test("OutcomeHelper constructs Success, Error, and Forward outcomes", () => {
  const resp = response("Hello");
  const succ = OutcomeHelper.success(resp);
  expect(succ.kind).toBe(OutcomeKind.Success);
  expect(succ.response?.status).toBe(200);

  const err = OutcomeHelper.error(400);
  expect(err.kind).toBe(OutcomeKind.Error);
  expect(err.status).toBe(400);

  const fwd = OutcomeHelper.forward(404);
  expect(fwd.kind).toBe(OutcomeKind.Forward);
  expect(fwd.status).toBe(404);
});

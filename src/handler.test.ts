import { expect, test } from "vite-plus/test";
import {
  outcomeSuccess,
  outcomeError,
  outcomeForward,
  HandlerOutcomeKind,
  response,
} from "../dist/index.js";

test("Handler outcome helpers construct correct outcome shapes", () => {
  const res = response("Success Body", { status: 200 });
  const succ = outcomeSuccess(res);
  expect(succ.kind).toBe(HandlerOutcomeKind.Success);
  expect(succ.response?.status).toBe(200);

  const err = outcomeError(403);
  expect(err.kind).toBe(HandlerOutcomeKind.Error);
  expect(err.status).toBe(403);

  const fwd = outcomeForward();
  expect(fwd.kind).toBe(HandlerOutcomeKind.Forward);
});

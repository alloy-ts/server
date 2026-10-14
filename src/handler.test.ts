import { expect, test } from "vite-plus/test";
import { OutcomeHelper, CustomHandlerInfo } from "../index.js";

test("handler.ts - OutcomeHelper outcome constructors", () => {
  const success = OutcomeHelper.success();
  expect(success.kind).toBe("success");
  expect(success.status).toBe(200);

  const error = OutcomeHelper.error(404);
  expect(error.kind).toBe("error");
  expect(error.status).toBe(404);

  const forward = OutcomeHelper.forward(500);
  expect(forward.kind).toBe("forward");
  expect(forward.status).toBe(500);
});

test("handler.ts - CustomHandlerInfo constructor", () => {
  const info = new CustomHandlerInfo("MyHandler", 3);
  expect(info.name).toBe("MyHandler");
  expect(info.routeCount).toBe(3);

  const defaultInfo = new CustomHandlerInfo("DefaultHandler");
  expect(defaultInfo.routeCount).toBe(1);
});

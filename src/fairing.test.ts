import { expect, test } from "vite-plus/test";
import { createFairingInfo } from "../dist/index.js";

test("FairingInfo creation and kind flag structure", () => {
  const info = createFairingInfo("Request Logger", {
    ignite: false,
    liftoff: true,
    request: true,
    response: true,
    shutdown: false,
    singleton: false,
  });

  expect(info.name).toBe("Request Logger");
  expect(info.kind.request).toBe(true);
  expect(info.kind.response).toBe(true);
  expect(info.kind.shutdown).toBe(false);
});

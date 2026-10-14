import { expect, test } from "vite-plus/test";
import { createFairingInfo } from "../dist/index.js";

test("createFairingInfo constructs FairingInfo object", () => {
  const info = createFairingInfo("Request Timer", {
    ignite: false,
    liftoff: false,
    request: true,
    response: true,
    shutdown: false,
    singleton: false,
  });

  expect(info.name).toBe("Request Timer");
  expect(info.kind.request).toBe(true);
  expect(info.kind.response).toBe(true);
  expect(info.kind.ignite).toBe(false);
});

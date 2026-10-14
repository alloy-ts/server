import { expect, test } from "vite-plus/test";
import { CustomFairing } from "../dist/index.js";

test("CustomFairing instantiates cleanly with name, kind, and liftoff callback", () => {
  let called = false;
  const fairing = new CustomFairing("TestFairing", { liftoff: true, request: true }, () => {
    called = true;
  });

  expect(fairing).toBeDefined();
  expect(called).toBe(false);
});

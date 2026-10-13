import { expect, test } from "vite-plus/test";
import { Alloy, serve } from "../dist/index.js";

test("Alloy and serve functions exist", () => {
  expect(typeof serve).toBe("function");
  expect(typeof Alloy.serve).toBe("function");
});

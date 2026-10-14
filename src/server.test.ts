import { expect, test } from "vite-plus/test";
import { file, launch, registerRoute, Response, serve } from "../dist/index.js";

test("server module exports Response, serve, file, registerRoute, and launch", () => {
  expect(typeof Response).toBe("function");
  expect(typeof serve).toBe("function");
  expect(typeof file).toBe("function");
  expect(typeof registerRoute).toBe("function");
  expect(typeof launch).toBe("function");
});

import { expect, test } from "vite-plus/test";
import { response } from "../dist/index.js";

test("handler response creates a valid NAPI response payload", () => {
  const res = response("OK", { status: 200 });
  expect(res.status).toBe(200);
  expect(res.body).toBeDefined();
});

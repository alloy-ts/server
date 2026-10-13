import { expect, test } from "vite-plus/test";
import { CODEGEN_DOC } from "./codegen.ts";

test("codegen documentation constant is defined", () => {
  expect(typeof CODEGEN_DOC).toBe("string");
  expect(CODEGEN_DOC).toContain("Rocket code generation documentation module");
});

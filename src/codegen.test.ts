import { expect, test } from "vite-plus/test";
import { getCodegenDoc, getCodegenInfo, isCodegenDebugEnabled } from "../dist/index.js";

test("codegen exports work as expected", () => {
  expect(typeof isCodegenDebugEnabled).toBe("function");
  expect(typeof isCodegenDebugEnabled()).toBe("boolean");
  expect(typeof getCodegenInfo).toBe("function");
  expect(getCodegenInfo()).toBe("Rocket Code Generation Module");
  expect(typeof getCodegenDoc).toBe("function");
  expect(getCodegenDoc()).toContain("Rocket Code Generation Facilities");
});

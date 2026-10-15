import { expect, test } from "vite-plus/test";
import { Compiler } from "../dist/index.js";

test("Compiler execute produces output given source text and path", () => {
  const compiler = new Compiler();
  const source = "const x: number = 42;";
  const output = compiler.execute(source, "ts", "src/index.ts");

  expect(output).toContain("src/index.ts");
  expect(output).toContain("const x: number = 42;");
});

test("Compiler handles empty source text", () => {
  const compiler = new Compiler();
  const output = compiler.execute("", "ts", "src/empty.ts");
  expect(output).toBe("");
});

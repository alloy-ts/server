import { expect, test } from "vite-plus/test";
import { main } from "../dist/index.js";

test("main returns Hello, world!", () => {
  expect(main()).toBe("Hello, world!");
});

import { expect, test } from "vite-plus/test";
import { uri } from "../dist/index.js";

test("uri replaces path parameters correctly", () => {
  expect(uri("/person/<name>", { name: "Mike" })).toBe("/person/Mike");
  expect(uri("/person/:name", { name: "Mike Smith" })).toBe("/person/Mike%20Smith");
});

test("uri handles query parameters", () => {
  expect(uri("/person/<name>?age", { name: "Mike" })).toBe("/person/Mike?age");
});

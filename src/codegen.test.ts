import { expect, test } from "vite-plus/test";
import { uri } from "../dist/index.js";

test("uri helper path parameter substitution (<id>, <id..>, :id)", () => {
  expect(uri("/users/<id>", { id: "42" })).toBe("/users/42");
  expect(uri("/files/<path..>", { path: "docs/readme.md" })).toBe("/files/docs%2Freadme.md");
  expect(
    uri("/posts/:postId/comments/:commentId", {
      postId: "10",
      commentId: "20",
    }),
  ).toBe("/posts/10/comments/20");
});

test("uri helper query parameter append and merge", () => {
  expect(uri("/search", undefined, { q: "rocket framework", page: "1" })).toContain(
    "q=rocket%20framework",
  );
  expect(uri("/search?category=code", undefined, { q: "napi" })).toBe(
    "/search?category=code&q=napi",
  );
});

test("uri helper percent encoding special characters", () => {
  expect(uri("/echo/<msg>", { msg: "hello world & rocket!" })).toBe(
    "/echo/hello%20world%20%26%20rocket%21",
  );
});

import { expect, test } from "vite-plus/test";
import { file, Response } from "../dist/index.js";

test("Response constructors and methods work as expected", () => {
  const res = new Response("Hello World", { status: 200, headers: { "X-Test": "1" } });
  expect(res.status).toBe(200);
  expect(res.headers["X-Test"]).toBe("1");

  const jsonRes = Response.json({ message: "ok" });
  expect(jsonRes.status).toBe(200);
  expect(jsonRes.headers["Content-Type"]).toBe("application/json");

  const redirectRes = Response.redirect("https://rocket.rs", 301);
  expect(redirectRes.status).toBe(301);
  expect(redirectRes.headers["Location"]).toBe("https://rocket.rs");
});

test("file helper works as expected", () => {
  const f = file("index.html");
  expect(f.path).toBe("index.html");
});

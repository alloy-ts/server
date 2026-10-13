import { expect, test } from "vite-plus/test";
import { file, launch, registerRoute, Response, serve, Server } from "./main.ts";

test("Server class and serve function", () => {
  expect(typeof serve).toBe("function");
  expect(typeof file).toBe("function");
  expect(typeof launch).toBe("function");
  expect(typeof registerRoute).toBe("function");
  expect(Response).toBeDefined();
  expect(Server).toBeDefined();
});

test("Static file helper", () => {
  const f = file("./test.txt");
  expect(f.path).toBe("./test.txt");
});

test("Response initialization and payload", () => {
  const r = new Response("Body Content", { status: 200, headers: { "X-Test": "true" } });
  expect(r.status).toBe(200);

  const redirectResp = Response.redirect("/target", 301);
  expect(redirectResp.status).toBe(301);
});

import { expect, test } from "vite-plus/test";
import { file, launch, registerRoute, Response, serve } from "./server.ts";

test("server module exports Response, serve, file, registerRoute, and launch", () => {
  expect(typeof serve).toBe("function");
  expect(typeof registerRoute).toBe("function");
  expect(typeof launch).toBe("function");
  expect(typeof file).toBe("function");
  expect(typeof Response).toBe("function");
});

test("server module Response helper creates responses", () => {
  const resp = new Response("server test");
  expect(resp.status).toBe(200);

  const jsonResp = Response.json({ server: true });
  expect(jsonResp.status).toBe(200);
});

test("server module file marker creates static file marker", () => {
  const marker = file("./static.txt");
  expect(marker.path).toBe("./static.txt");
});

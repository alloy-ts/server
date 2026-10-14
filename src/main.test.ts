import { expect, test } from "vite-plus/test";
import { Response, serve, uri } from "../index.js";

test("Response constructor and JSON creation", () => {
  const resp = new Response("Hello");
  expect(resp.status).toBe(200);

  const jsonResp = Response.json({ status: "ok" });
  expect(jsonResp.status).toBe(200);
});

test("serve and uri export availability", () => {
  expect(typeof serve).toBe("function");
  expect(typeof uri).toBe("function");
});

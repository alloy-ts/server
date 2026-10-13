import { expect, test } from "vite-plus/test";
import {
  main,
  Alloy,
  response,
  responseJson,
  responseRedirect,
  parseRouteUri,
  generateTsUriBuilder,
} from "./main.ts";

test("main returns Hello, world!", () => {
  expect(main()).toBe("Hello, world!");
});

test("Alloy serve and file helpers are functions", () => {
  expect(typeof Alloy.serve).toBe("function");
  expect(typeof Alloy.file).toBe("function");
});

test("Response constructors produce expected outputs", () => {
  const resp = response("ok", { status: 200 });
  expect(resp.status).toBe(200);

  const jsonResp = responseJson({ ok: true });
  expect(jsonResp.status).toBe(200);
  expect(jsonResp.headers["Content-Type"]).toBe("application/json");

  const redirResp = responseRedirect("https://example.com", 301);
  expect(redirResp.status).toBe(301);
  expect(redirResp.headers["Location"]).toBe("https://example.com");
});

test("Codegen metadata parsing works", () => {
  const meta = parseRouteUri("/posts/:slug", "GET");
  expect(meta.rocketPath).toBe("/posts/<slug>");
  expect(meta.params.length).toBe(1);
  expect(meta.params[0].name).toBe("slug");

  const tsCode = generateTsUriBuilder([{ name: "getPost", method: "GET", path: "/posts/:slug" }]);
  expect(tsCode).toContain("getPost: (slug: string | number) => `/posts/${slug}`");
});

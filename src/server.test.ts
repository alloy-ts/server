import { expect, test } from "vite-plus/test";
import { file, registerRoute, serve, response, responseJson } from "../dist/index.js";

test("server module exports serve, registerRoute, file, response, responseJson", () => {
  expect(typeof serve).toBe("function");
  expect(typeof registerRoute).toBe("function");
  expect(typeof file).toBe("function");
  expect(typeof response).toBe("function");
  expect(typeof responseJson).toBe("function");
});

test("server module response helpers create responses", () => {
  const resp = response("server test");
  expect(resp.status).toBe(200);

  const jsonResp = responseJson({ server: true });
  expect(jsonResp.status).toBe(200);
});

test("server module file marker creates static file marker", () => {
  const marker = file("./static.txt");
  expect(marker.path).toBe("./static.txt");
});

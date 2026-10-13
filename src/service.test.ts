import { expect, test } from "vite-plus/test";
import {
  file,
  registerRoute,
  serve,
  response,
  responseJson,
} from "../dist/index.js";

test("service module exports serve, registerRoute, file, response, responseJson", () => {
  expect(typeof serve).toBe("function");
  expect(typeof registerRoute).toBe("function");
  expect(typeof file).toBe("function");
  expect(typeof response).toBe("function");
  expect(typeof responseJson).toBe("function");
});

test("service module response helpers create responses", () => {
  const resp = response("service test");
  expect(resp.status).toBe(200);

  const jsonResp = responseJson({ service: true });
  expect(jsonResp.status).toBe(200);
});

test("service module file marker creates static file marker", () => {
  const marker = file("./static.txt");
  expect(marker.path).toBe("./static.txt");
});

import { expect, test } from "vite-plus/test";
import {
  Alloy,
  App,
  Bun,
  file,
  registerRoute,
  serve,
  response,
  responseJson,
  responseRedirect,
} from "../dist/index.js";

test("server module exports serve, Alloy.serve, Bun.serve, App, registerRoute, file, response helpers", () => {
  expect(typeof serve).toBe("function");
  expect(typeof Alloy.serve).toBe("function");
  expect(typeof Bun.serve).toBe("function");
  expect(typeof App).toBe("function");
  expect(typeof Alloy.file).toBe("function");
  expect(typeof Bun.file).toBe("function");
  expect(typeof registerRoute).toBe("function");
  expect(typeof file).toBe("function");
  expect(typeof response).toBe("function");
  expect(typeof responseJson).toBe("function");
  expect(typeof responseRedirect).toBe("function");
});

test("server module response helpers create responses", () => {
  const resp = response("server test");
  expect(resp.status).toBe(200);

  const jsonResp = responseJson({ server: true });
  expect(jsonResp.status).toBe(200);

  const redirectResp = responseRedirect("/target");
  expect(redirectResp.status).toBe(302);
  expect(redirectResp.headers["location"]).toBe("/target");
});

test("server module file marker creates static file marker", () => {
  const marker = file("./static.txt");
  expect(marker.path).toBe("./static.txt");

  const alloyMarker = Alloy.file("./favicon.ico");
  expect(alloyMarker.path).toBe("./favicon.ico");

  const bunMarker = Bun.file("./favicon.ico");
  expect(bunMarker.path).toBe("./favicon.ico");
});

test("Alloy.serve starts a server and returns port/url", () => {
  const server = Alloy.serve({
    port: 0,
    routes: {
      "/api/status": response("OK"),
    },
  });

  expect(typeof server.port).toBe("number");
  expect(server.port).toBeGreaterThan(0);
  expect(server.url).toContain(`http://localhost:${server.port}`);
  server.stop();
});

test("App class registers routes and launches server", () => {
  const app = new App();
  app.get("/", async () => response("Hello, App!"));

  const server = app.launch({ port: 0 });
  expect(typeof server.port).toBe("number");
  expect(server.url).toContain(`http://localhost:${server.port}`);
  app.stop();
});

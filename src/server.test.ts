import { expect, test } from "vite-plus/test";
import {
  Alloy,
  App,
  Bun,
  file,
  registerRoute,
  response,
  responseJson,
  responseRedirect,
  serve,
} from "../dist/index.js";

test("server module exports App, Alloy, Bun, serve, file, registerRoute, and response helpers", () => {
  expect(typeof App).toBe("function");
  expect(typeof serve).toBe("function");
  expect(typeof Alloy.serve).toBe("function");
  expect(typeof Bun.serve).toBe("function");
  expect(typeof Alloy.file).toBe("function");
  expect(typeof Bun.file).toBe("function");
  expect(typeof registerRoute).toBe("function");
  expect(typeof file).toBe("function");
  expect(typeof response).toBe("function");
  expect(typeof responseJson).toBe("function");
  expect(typeof responseRedirect).toBe("function");
});

test("server response helpers create valid Response objects", () => {
  const resp = response("server test");
  expect(resp.status).toBe(200);

  const jsonResp = responseJson({ server: true });
  expect(jsonResp.status).toBe(200);
  expect(jsonResp.headers["Content-Type"]).toBe("application/json");

  const redirResp = responseRedirect("/blog/hello/world", 302);
  expect(redirResp.status).toBe(302);
  expect(redirResp.headers["Location"]).toBe("/blog/hello/world");
});

test("server module file marker creates static file marker", () => {
  const marker = file("./static.txt");
  expect(marker.path).toBe("./static.txt");

  const alloyMarker = Alloy.file("./static.txt");
  expect(alloyMarker.path).toBe("./static.txt");

  const bunMarker = Bun.file("./static.txt");
  expect(bunMarker.path).toBe("./static.txt");
});

test("Alloy.serve accepts options object and resolves port", () => {
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

test("App builder configures and launches server", () => {
  const app = new App();
  app.get("/", async () => response("Hello, world!"));
  const server = app.launch({ port: 0 });

  expect(typeof server.port).toBe("number");
  expect(server.port).toBeGreaterThan(0);
  expect(server.url).toContain(`http://localhost:${server.port}`);
  server.stop();
});

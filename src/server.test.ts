import { expect, test } from "vite-plus/test";
import {
  serve,
  response,
  responseJson,
  responseRedirect,
  file,
  registerRoute,
  App,
} from "../dist/index.js";

test("response helpers create valid response payloads", () => {
  const textResp = response("Hello Server", { status: 200 });
  expect(textResp.status).toBe(200);

  const jsonResp = responseJson({ ok: true });
  expect(jsonResp.status).toBe(200);
  expect(jsonResp.headers["Content-Type"]).toBe("application/json");

  const redirectResp = responseRedirect("/target", 301);
  expect(redirectResp.status).toBe(301);
  expect(redirectResp.headers.Location).toBe("/target");
});

test("file helper creates StaticFile object", () => {
  const sf = file("./assets/logo.png");
  expect(sf.path).toBe("./assets/logo.png");
});

test("serve initializes server with routes object and options", () => {
  const server = serve({
    port: 0,
    routes: {
      "/status": response("OK"),
    },
  });
  expect(server.port).toBeGreaterThan(0);
  expect(server.url).toContain("http://");
  server.stop();
});

test("registerRoute adds routes to low-level route registry", () => {
  expect(() => {
    registerRoute("GET", "/registered", async () => response("registered"));
  }).not.toThrow();
});

test("App class handles get, post, put, delete, patch route registration and launch", () => {
  const app = new App({ port: 0 });
  app.get("/get", async () => response("get"));
  app.post("/post", async () => response("post"));
  app.put("/put", async () => response("put"));
  app.delete("/delete", async () => response("delete"));
  app.patch("/patch", async () => response("patch"));

  const server = app.launch();
  expect(server.port).toBeGreaterThan(0);
  expect(server.url).toContain("http://");
  server.stop();
});

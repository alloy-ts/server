import { expect, test } from "vite-plus/test";
import { response, file, serve, uri, App, main } from "../dist/index.js";

test("main returns Hello, world!", () => {
  expect(main()).toBe("Hello, world!");
});

test("uri constructs type-safe URIs with dynamic parameter encoding", () => {
  const generatedUri = uri("/users/:id/posts/:post_id", {
    id: "42",
    post_id: "hello world",
  });
  expect(generatedUri).toBe("/users/42/posts/hello%20world");
});

test("response constructors create valid responses", () => {
  const resp = response("Hello", { status: 200 });
  expect(resp.status).toBe(200);
});

test("file returns StaticFile marker", () => {
  const staticFile = file("./favicon.ico");
  expect(staticFile.path).toBe("./favicon.ico");
});

test("serve starts and stops server", () => {
  const server = serve({
    port: 0,
    routes: {
      "/api/status": response("OK"),
    },
  });
  expect(server.port).toBeGreaterThan(0);
  expect(server.url).toContain("http://");
  server.stop();
});

test("App class registers routes and launches server", () => {
  const app = new App({ port: 0 });
  app.get("/hello", async () => response("world"));
  const server = app.launch();
  expect(server.port).toBeGreaterThan(0);
  expect(server.url).toContain("http://");
  server.stop();
});

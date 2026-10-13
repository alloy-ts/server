import { expect, test } from "vite-plus/test";
import { Alloy, main } from "./main.ts";

test("main returns Hello, world!", () => {
  expect(main()).toBe("Hello, world!");
});

test("Alloy.uri constructs type-safe URIs with dynamic parameter encoding", () => {
  const uri = Alloy.uri("/users/:id/posts/:post_id", {
    id: "42",
    post_id: "hello world",
  });
  expect(uri).toBe("/users/42/posts/hello%20world");
});

test("Alloy.Response constructors create valid responses", () => {
  const resp = new Alloy.Response("Hello", { status: 200 });
  expect(resp.status).toBe(200);

  const jsonResp = Alloy.Response.json({ success: true });
  expect(jsonResp.status).toBe(200);

  const redirectResp = Alloy.Response.redirect("/target");
  expect(redirectResp.status).toBe(302);
});

test("Alloy.file returns StaticFile marker", () => {
  const staticFile = Alloy.file("./favicon.ico");
  expect(staticFile.path).toBe("./favicon.ico");
});

test("Alloy.serve starts and stops server", () => {
  const server = Alloy.serve({
    port: 0,
    routes: {
      "/api/status": new Alloy.Response("OK"),
    },
  });
  expect(server.port).toBeGreaterThan(0);
  expect(server.url).toContain("http://");
  server.stop();
});

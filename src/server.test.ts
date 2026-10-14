import { expect, test } from "vite-plus/test";
import {
  serve,
  file,
  Response,
  registerRoute,
  parseRoutePattern,
  buildRouteUri,
} from "../index.js";

test("server.ts - Response constructor and static helpers", () => {
  const resp = new Response("ok", { status: 200, headers: [["Content-Type", "text/plain"]] });
  expect(resp.status).toBe(200);

  const jsonResp = Response.json({ key: "value" });
  expect(jsonResp.status).toBe(200);

  const redirectResp = Response.redirect("https://example.com");
  expect(redirectResp.status).toBe(302);
});

test("server.ts - file() helper", () => {
  const staticFile = file("./static/index.html");
  expect(staticFile.path).toBe("./static/index.html");
});

test("server.ts - registerRoute accumulates route definitions", () => {
  expect(() => {
    registerRoute("GET", "/test-path", () => Response.json({ ok: true }));
  }).not.toThrow();
});

test("server.ts - serve() initializes Rocket server instance", () => {
  const server = serve({
    port: 0,
    hostname: "localhost",
    routes: {
      "/health": () => new Response("OK"),
    },
    fetch: () => new Response("Not Found", { status: 404 }),
  });

  expect(server.url).toContain("http://localhost:");
  expect(typeof server.port).toBe("number");

  expect(() => server.stop()).not.toThrow();
});

test("server.ts - pattern translation and URI building integration", () => {
  const parsed = parseRoutePattern("/items/:id");
  expect(parsed.rocketPattern).toBe("/items/<id>");

  const uri = buildRouteUri("/items/:id", { id: "42" });
  expect(uri).toBe("/items/42");
});

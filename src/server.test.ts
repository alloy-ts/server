import { expect, test } from "vite-plus/test";
import { Alloy, Config, file, Response, serve } from "./main.ts";

test("Response creates responses with status, headers, and body", () => {
  const resp = new Response("hello world", {
    status: 200,
    headers: [["X-Custom-Header", "test-value"]],
  });
  expect(resp.status).toBe(200);
  expect(resp.headers).toEqual([["X-Custom-Header", "test-value"]]);

  const jsonResp = Response.json({ success: true, count: 5 });
  expect(jsonResp.status).toBe(200);
  expect(jsonResp.headers).toContainEqual(["Content-Type", "application/json"]);

  const redirectResp = Response.redirect("/dashboard", 307);
  expect(redirectResp.status).toBe(307);
  expect(redirectResp.headers).toEqual([["Location", "/dashboard"]]);
});

test("Config default profiles, settings, and parameter list", () => {
  const cfg = new Config();
  expect(cfg.port).toBeGreaterThan(0);
  expect(cfg.logLevel).toBeDefined();

  const debugCfg = Config.debugDefault();
  expect(debugCfg.profile).toBe("debug");

  const releaseCfg = Config.releaseDefault();
  expect(releaseCfg.profile).toBe("release");

  const params = Config.parameters();
  expect(params).toContain("port");
  expect(params).toContain("address");
  expect(params).toContain("workers");
  expect(params).toContain("shutdown");
});

test("file helper creates static file definition", () => {
  const staticFile = file("./assets/logo.png");
  expect(staticFile.path).toBe("./assets/logo.png");
});

test("Alloy.serve launches Rocket server instance and allows graceful stop", () => {
  const server = Alloy.serve({
    port: 0,
    hostname: "127.0.0.1",
    routes: {
      "/health": new Response("OK"),
      "/users/:id": (req: any) => new Response(`User ${req.params.id}`),
      "/static.txt": file("./test.txt"),
    },
    fetch() {
      return new Response("Fallback Not Found", { status: 404 });
    },
  });

  expect(serve).toBeDefined();
  expect(server.url).toContain("http://127.0.0.1:");
  expect(server.port).toBeGreaterThanOrEqual(0);

  expect(() => server.stop()).not.toThrow();
});

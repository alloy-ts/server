import { expect, test } from "vite-plus/test";
import { Alloy, Config, file, Response, serve } from "../dist/index.js";

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

test("Alloy.serve supports Bun.serve compatible setup", () => {
  const server = Alloy.serve({
    port: 0,
    hostname: "127.0.0.1",
    routes: {
      // Static route
      "/api/status": new Response("OK"),

      // Dynamic route
      "/users/:id": (req: any) => new Response(`Hello User ${req.params.id}!`),

      // Per-HTTP method handlers
      "/api/posts": {
        GET: () => new Response("List posts"),
        POST: async (req: any) => {
          return Response.json({ created: true, body: req.body });
        },
      },

      // Wildcard route
      "/api/*": Response.json({ message: "Not found" }, { status: 404 }),

      // Redirect
      "/blog/hello": Response.redirect("/blog/hello/world"),

      // File
      "/favicon.ico": Alloy.file("./favicon.ico"),
    },

    // Fallback for unmatched routes
    fetch() {
      return new Response("Not Found", { status: 404 });
    },
  });

  expect(server.url).toContain("http://127.0.0.1:");
  expect(server.port).toBeGreaterThanOrEqual(0);

  expect(() => server.stop()).not.toThrow();
});

test("serve resolves port from environment variables", () => {
  process.env["BUN_PORT"] = "4002";
  const server = serve();
  expect(server.port).toBe(4002);
  server.stop();
  delete process.env["BUN_PORT"];
});

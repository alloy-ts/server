import { expect, test } from "vite-plus/test";
import { Config, Response, file, serve } from "./main.ts";

test("Config initializes, getters/setters work, and tryFrom creates custom configs", () => {
  const config = new Config();
  expect(config.port).toBe(8000);
  expect(config.address).toBe("127.0.0.1");

  config.port = 9000;
  expect(config.port).toBe(9000);

  config.address = "0.0.0.0";
  expect(config.address).toBe("0.0.0.0");

  const custom = Config.tryFrom({ port: 4000, address: "127.0.0.1" });
  expect(custom.port).toBe(4000);
});

test("Response constructs status, headers, and body for text, json, and redirects", () => {
  const text = new Response("Plain Text", { status: 200, headers: { "X-Test": "1" } });
  expect(text.status).toBe(200);
  expect(text.headers["X-Test"]).toBe("1");
  expect(text.body?.toString()).toBe("Plain Text");

  const json = Response.json({ success: true });
  expect(json.status).toBe(200);
  expect(json.headers["Content-Type"]).toBe("application/json");
  expect(JSON.parse(json.body?.toString() || "{}")).toEqual({ success: true });

  const redirect = Response.redirect("https://example.com", 301);
  expect(redirect.status).toBe(301);
  expect(redirect.headers["Location"]).toBe("https://example.com");
});

test("file helper creates static file markers", () => {
  const staticFile = file("./static/index.html");
  expect(staticFile.path).toBe("./static/index.html");
});

test("serve initializes Rocket server instance with url and stop method", () => {
  const server = serve({
    port: 0,
    hostname: "127.0.0.1",
    routes: {
      "/health": () => new Response("OK"),
    },
  });

  expect(server.url).toContain("http://127.0.0.1:");
  expect(typeof server.port).toBe("number");

  expect(() => server.stop()).not.toThrow();
});

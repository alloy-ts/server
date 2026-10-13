import { expect, test } from "vite-plus/test";
import { Alloy, file, registerRoute, response, responseJson, responseRedirect } from "./main.ts";

test("response constructors generate expected response objects", () => {
  const r1 = response("Hello world", {
    status: 200,
    headers: { "X-Test": "1" },
  });
  expect(r1.status).toBe(200);
  expect(r1.headers["X-Test"]).toBe("1");

  const r2 = responseJson({ ok: true }, { status: 201 });
  expect(r2.status).toBe(201);
  expect(r2.headers["Content-Type"]).toBe("application/json");

  const r3 = responseRedirect("https://example.com", 301);
  expect(r3.status).toBe(301);
  expect(r3.headers["Location"]).toBe("https://example.com");
});

test("Alloy.file and file function return static file specifier", () => {
  const f1 = file("./test.txt");
  expect(f1.path).toBe("./test.txt");

  const f2 = Alloy.file("./favicon.ico");
  expect(f2.path).toBe("./favicon.ico");
});

test("Alloy.serve starts a Rocket server instance that can be stopped", async () => {
  const server = Alloy.serve({
    port: 0,
    routes: {
      "/api/health": responseJson({ status: "ok" }),
      "/users/:id": (req: any) => response(`User ${req.params.id}`),
    },
    fetch() {
      return response("Not Found", { status: 404 });
    },
  });

  expect(server.url).toContain("http://");
  expect(server.port).toBeGreaterThanOrEqual(0);

  // Stop server gracefully
  server.stop();
});

test("registerRoute accumulates routes prior to server launch", () => {
  expect(() => {
    registerRoute("GET", "/registered/route", async () => response("OK"));
  }).not.toThrow();
});

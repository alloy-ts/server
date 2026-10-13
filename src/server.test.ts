import { describe, expect, it } from "vite-plus/test";
import {
  Alloy,
  file,
  registerRoute,
  type RequestInput,
  response,
  responseJson,
  responseRedirect,
  serve,
} from "./main.ts";

describe("server NAPI bindings", () => {
  it("response helper constructs Response object", () => {
    const res = response("Hello world", {
      status: 201,
      headers: { "x-custom": "value" },
    });

    expect(res.status).toBe(201);
    expect(res.headers["x-custom"]).toBe("value");
    expect(res.body).toBeDefined();

    const text = new TextDecoder().decode(Uint8Array.from(res.body ?? []));
    expect(text).toBe("Hello world");
  });

  it("responseJson constructs JSON Response with headers", () => {
    const res = responseJson({ status: "ok", count: 42 });

    expect(res.status).toBe(200);
    expect(res.headers["Content-Type"]).toBe("application/json");

    const json = JSON.parse(new TextDecoder().decode(Uint8Array.from(res.body ?? [])));
    expect(json).toEqual({ status: "ok", count: 42 });
  });

  it("responseRedirect constructs redirect Response", () => {
    const res = responseRedirect("https://example.com/login", 301);

    expect(res.status).toBe(301);
    expect(res.headers["Location"]).toBe("https://example.com/login");
    expect(res.body).toBeUndefined();
  });

  it("file helper constructs StaticFile object", () => {
    const sf = file("./static/app.js");
    expect(sf.path).toBe("./static/app.js");
  });

  it("Alloy.serve supports single options object signature and Alloy namespace", async () => {
    const port = 3012;
    const server = Alloy.serve({
      port,
      hostname: "127.0.0.1",
      routes: {
        "/api/status": response("OK"),
        "/users/:id": (req: RequestInput) => response(`Hello User ${req.params["id"]}!`),
        "/api/posts": {
          GET: () => response("List posts"),
          POST: async (req: RequestInput) => {
            const bodyText = req.body ? new TextDecoder().decode(Uint8Array.from(req.body)) : "";
            return responseJson({ created: true, received: bodyText });
          },
        },
        "/api/*": responseJson({ message: "Not found" }, { status: 404 }),
        "/blog/hello": responseRedirect("/blog/hello/world"),
        "/favicon.ico": Alloy.file("./favicon.ico"),
      },
      fetch(req: RequestInput) {
        return response(`Fallback for ${req.path}`, { status: 404 });
      },
    });

    expect(server.port).toBe(port);
    expect(server.url).toBe(`http://127.0.0.1:${port}`);

    await new Promise((r) => setTimeout(r, 200));

    // Test static route
    const res1 = await fetch(`http://127.0.0.1:${port}/api/status`);
    expect(res1.status).toBe(200);
    expect(await res1.text()).toBe("OK");

    // Test dynamic route
    const res2 = await fetch(`http://127.0.0.1:${port}/users/123`);
    expect(res2.status).toBe(200);
    expect(await res2.text()).toBe("Hello User 123!");

    // Test per-method GET
    const res3 = await fetch(`http://127.0.0.1:${port}/api/posts`);
    expect(res3.status).toBe(200);
    expect(await res3.text()).toBe("List posts");

    // Test redirect
    const res4 = await fetch(`http://127.0.0.1:${port}/blog/hello`, {
      redirect: "manual",
    });
    expect(res4.status).toBe(302);
    expect(res4.headers.get("Location")).toBe("/blog/hello/world");

    server.stop();
  });

  it("serve respects PORT environment variable fallback when port option is omitted", async () => {
    process.env["PORT"] = "3013";
    try {
      const server = Alloy.serve({
        hostname: "127.0.0.1",
        routes: {
          "/ping": response("pong"),
        },
      });

      expect(server.port).toBe(3013);
      await new Promise((r) => setTimeout(r, 200));

      const res = await fetch(`http://127.0.0.1:3013/ping`);
      expect(res.status).toBe(200);
      expect(await res.text()).toBe("pong");

      server.stop();
    } finally {
      delete process.env["PORT"];
    }
  });

  it("serve boots server with 3-argument signature and handles routes and stops", async () => {
    const port = 3010;
    const server = serve(
      {
        "/api/status": response("OK"),
        "/users/:id": (req: RequestInput) => response(`User ${req.params["id"]}`),
      },
      (req: RequestInput) => response(`Fallback for ${req.path}`, { status: 404 }),
      { port, hostname: "127.0.0.1" },
    );

    expect(server.port).toBe(port);
    await new Promise((r) => setTimeout(r, 200));

    const res = await fetch(`http://127.0.0.1:${port}/api/status`);
    expect(res.status).toBe(200);
    expect(await res.text()).toBe("OK");

    server.stop();
  });

  it("registerRoute registers routes programmatically", async () => {
    const port = 3011;
    registerRoute("GET", "/registered/ping", () => response("pong"));

    const server = serve(null, null, { port, hostname: "127.0.0.1" });
    await new Promise((r) => setTimeout(r, 200));

    const res = await fetch(`http://127.0.0.1:${port}/registered/ping`);
    expect(res.status).toBe(200);
    expect(await res.text()).toBe("pong");

    server.stop();
  });
});

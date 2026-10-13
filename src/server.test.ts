import { describe, expect, it } from "vite-plus/test";
import {
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

  it("serve boots server, handles sync/async routes, params, and stops", async () => {
    const port = 3010;
    const server = serve(
      {
        "/api/status": response("OK"),
        "/users/:id": (req: RequestInput) => response(`User ${req.params["id"]}`),
        "/api/items": {
          GET: () => responseJson(["item1", "item2"]),
          POST: async (req: RequestInput) => {
            const bodyText = req.body ? new TextDecoder().decode(Uint8Array.from(req.body)) : "";
            return responseJson({ created: true, received: bodyText });
          },
        },
      },
      (req: RequestInput) => response(`Fallback for ${req.path}`, { status: 404 }),
      { port, hostname: "127.0.0.1" },
    );

    expect(server.port).toBe(port);
    expect(server.url).toBe(`http://127.0.0.1:${port}`);

    // Wait briefly for Rocket liftoff
    await new Promise((r) => setTimeout(r, 200));

    // Test static route
    const res1 = await fetch(`http://127.0.0.1:${port}/api/status`);
    expect(res1.status).toBe(200);
    expect(await res1.text()).toBe("OK");

    // Test sync route with path params
    const res2 = await fetch(`http://127.0.0.1:${port}/users/42`);
    expect(res2.status).toBe(200);
    expect(await res2.text()).toBe("User 42");

    // Test per-method GET
    const res3 = await fetch(`http://127.0.0.1:${port}/api/items`);
    expect(res3.status).toBe(200);
    expect(await res3.json()).toEqual(["item1", "item2"]);

    // Test per-method POST async with body
    const res4 = await fetch(`http://127.0.0.1:${port}/api/items`, {
      method: "POST",
      body: "payload-data",
    });
    expect(res4.status).toBe(200);
    expect(await res4.json()).toEqual({
      created: true,
      received: "payload-data",
    });

    // Test fallback fetch
    const res5 = await fetch(`http://127.0.0.1:${port}/unknown/path`);
    expect(res5.status).toBe(404);
    expect(await res5.text()).toBe("Fallback for /unknown/path");

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

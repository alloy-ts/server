import { expect, test } from "vite-plus/test";
import {
  file,
  registerRoute,
  response,
  responseJson,
  responseRedirect,
  serve,
} from "../dist/index.js";

test("server module exports response, serve, file, and registerRoute", () => {
  expect(typeof serve).toBe("function");
  expect(typeof registerRoute).toBe("function");
  expect(typeof file).toBe("function");
  expect(typeof response).toBe("function");
  expect(typeof responseJson).toBe("function");
  expect(typeof responseRedirect).toBe("function");
});

test("response helper creates Response objects with status and headers", () => {
  const resp = response("hello world", {
    status: 201,
    headers: { "x-custom": "test" },
  });
  expect(resp.status).toBe(201);
  expect(resp.headers["x-custom"]).toBe("test");
  expect(resp.body).toBeDefined();

  const jsonResp = responseJson({ ok: true });
  expect(jsonResp.status).toBe(200);
  expect(jsonResp.headers["Content-Type"]).toBe("application/json");

  const redirResp = responseRedirect("/login", 301);
  expect(redirResp.status).toBe(301);
  expect(redirResp.headers["Location"]).toBe("/login");
});

test("file helper creates StaticFile marker object", () => {
  const marker = file("./public/index.html");
  expect(marker.path).toBe("./public/index.html");
});

test("serve starts Rocket HTTP server and handles HTTP requests", async () => {
  const port = 3098;
  const server = serve(
    {
      "/api/status": response("OK"),
      "/api/echo": async (req: any) => {
        const bodyStr = req.body ? new TextDecoder().decode(new Uint8Array(req.body)) : "";
        return responseJson({
          method: req.method,
          path: req.path,
          query: req.query,
          headers: req.headers,
          bodyStr,
        });
      },
      "/users/:id": async (req: any) => response(`User:${req.params.id}`),
      "/files/*": async (req: any) => response(`File:${req.params._}`),
      "/methods": {
        GET: async () => response("GOT METHOD"),
        POST: async () => response("POSTED METHOD", { status: 201 }),
      },
    },
    async () => response("404 Not Found", { status: 404 }),
    { port, hostname: "127.0.0.1" },
  );

  expect(server.port).toBe(port);
  expect(server.url).toBe(`http://127.0.0.1:${port}`);

  await new Promise((r) => setTimeout(r, 200));

  try {
    // 1. Static response
    const res1 = await fetch(`${server.url}/api/status`);
    expect(res1.status).toBe(200);
    expect(await res1.text()).toBe("OK");

    // 2. Request details, query, headers, and body
    const res2 = await fetch(`${server.url}/api/echo?version=1.0`, {
      method: "POST",
      headers: { "Content-Type": "text/plain", "x-client": "test-suite" },
      body: "payload data",
    });
    expect(res2.status).toBe(200);
    const echoJson = (await res2.json()) as {
      method: string;
      path: string;
      query: string;
      headers: Record<string, string>;
      bodyStr: string;
    };
    expect(echoJson.method).toBe("POST");
    expect(echoJson.path).toBe("/api/echo");
    expect(echoJson.query).toBe("version=1.0");
    expect(echoJson.headers["x-client"]).toBe("test-suite");
    expect(echoJson.bodyStr).toBe("payload data");

    // 3. Single parameter extraction
    const res3 = await fetch(`${server.url}/users/123`);
    expect(res3.status).toBe(200);
    expect(await res3.text()).toBe("User:123");

    // 4. Wildcard multi-segment parameter extraction
    const res4 = await fetch(`${server.url}/files/docs/readme.txt`);
    expect(res4.status).toBe(200);
    expect(await res4.text()).toBe("File:docs/readme.txt");

    // 5. Per-method map (GET)
    const res5Get = await fetch(`${server.url}/methods`);
    expect(res5Get.status).toBe(200);
    expect(await res5Get.text()).toBe("GOT METHOD");

    // 6. Per-method map (POST)
    const res5Post = await fetch(`${server.url}/methods`, { method: "POST" });
    expect(res5Post.status).toBe(201);
    expect(await res5Post.text()).toBe("POSTED METHOD");

    // 7. Fallback fetch handler
    const res404 = await fetch(`${server.url}/nonexistent`);
    expect(res404.status).toBe(404);
    expect(await res404.text()).toBe("404 Not Found");
  } finally {
    server.stop();
  }
});

test("registerRoute registers custom route before serve", async () => {
  registerRoute("GET", "/registered/route", async () => response("Registered Handler"));

  const port = 3097;
  const server = serve({}, undefined, { port, hostname: "127.0.0.1" });

  await new Promise((r) => setTimeout(r, 200));

  try {
    const res = await fetch(`${server.url}/registered/route`);
    expect(res.status).toBe(200);
    expect(await res.text()).toBe("Registered Handler");
  } finally {
    server.stop();
  }
});

import { expect, test } from "vite-plus/test";
import { serve, Response, Config } from "../dist/index.js";

test("server response methods", () => {
  const resp = new Response("hello world", { status: 200 });
  expect(resp.status).toBe(200);

  const jsonResp = Response.json({ key: "value" }, { status: 201 });
  expect(jsonResp.status).toBe(201);

  const redirectResp = Response.redirect("https://rocket.rs", 302);
  expect(redirectResp.status).toBe(302);
});

test("server config initialization", () => {
  const cfg = Config.debugDefault();
  expect(cfg.port).toBeDefined();
  expect(cfg.address).toBeDefined();
});

test("serve handles static, dynamic, per-method, and fallback routes", async () => {
  const port = 4000 + Math.floor(Math.random() * 4000);
  const server = serve({
    port,
    hostname: "127.0.0.1",
    routes: {
      "/api/status": new Response("OK"),
      "/users/:id": (req: any) => {
        return new Response(`User ${req.params?.id || "unknown"}`);
      },
      "/api/posts": {
        GET: () => Response.json([{ id: 1, title: "Post 1" }]),
        POST: async () => {
          return Response.json({ created: true }, { status: 201 });
        },
      },
    },
    fetch() {
      return new Response("Fallback Not Found", { status: 404 });
    },
  });

  expect(server.url).toBe(`http://127.0.0.1:${port}`);
  expect(server.port).toBe(port);

  await new Promise((r) => setTimeout(r, 200));

  try {
    const statusRes = await fetch(`${server.url}/api/status`);
    expect(statusRes.status).toBe(200);
    expect(await statusRes.text()).toBe("OK");

    const userRes = await fetch(`${server.url}/users/42`);
    expect(userRes.status).toBe(200);
    expect(await userRes.text()).toBe("User 42");

    const postsGetRes = await fetch(`${server.url}/api/posts`);
    expect(postsGetRes.status).toBe(200);
    expect(await postsGetRes.json()).toEqual([{ id: 1, title: "Post 1" }]);

    const postsPostRes = await fetch(`${server.url}/api/posts`, {
      method: "POST",
    });
    expect(postsPostRes.status).toBe(201);
    expect(await postsPostRes.json()).toEqual({ created: true });

    const fallbackRes = await fetch(`${server.url}/unknown/path`);
    expect(fallbackRes.status).toBe(404);
    expect(await fallbackRes.text()).toBe("Fallback Not Found");
  } finally {
    server.stop();
  }
});

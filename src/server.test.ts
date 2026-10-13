import { expect, test } from "vite-plus/test";
import {
  Alloy,
  App,
  file,
  registerRoute,
  Response,
  response,
  responseJson,
  responseRedirect,
  serve,
} from "../dist/index.js";

test("server module exports Response, Alloy, App, serve, file, and registerRoute", () => {
  expect(typeof serve).toBe("function");
  expect(typeof registerRoute).toBe("function");
  expect(typeof file).toBe("function");
  expect(typeof response).toBe("function");
  expect(typeof responseJson).toBe("function");
  expect(typeof responseRedirect).toBe("function");
  expect(typeof Alloy.serve).toBe("function");
  expect(typeof Alloy.file).toBe("function");
  expect(typeof App).toBe("function");
  expect(typeof Response).toBe("function");
  expect(typeof Response.json).toBe("function");
  expect(typeof Response.redirect).toBe("function");
});

test("Response class constructor and factory methods create valid responses", () => {
  const resp = new Response("hello world", {
    status: 201,
    headers: { "x-custom": "test" },
  });
  expect(resp.status).toBe(201);
  expect(resp.headers["x-custom"]).toBe("test");

  const jsonResp = Response.json({ ok: true });
  expect(jsonResp.status).toBe(200);
  expect(jsonResp.headers["Content-Type"]).toBe("application/json");

  const redirResp = Response.redirect("/login", 301);
  expect(redirResp.status).toBe(301);
  expect(redirResp.headers["Location"]).toBe("/login");
});

test("file helper creates StaticFile marker object", () => {
  const marker = file("./public/index.html");
  expect(marker.path).toBe("./public/index.html");
});

test("Alloy.serve starts Rocket server with Bun.serve single-object options", async () => {
  const port = 3096;
  const server = Alloy.serve({
    port,
    hostname: "127.0.0.1",
    routes: {
      "/api/status": new Response("OK"),
      "/users/:id": (req: any) => new Response(`Hello User ${req.params.id}!`),
      "/api/posts": {
        GET: () => new Response("List posts"),
        POST: async (req: any) => {
          const bodyStr = req.body ? new TextDecoder().decode(new Uint8Array(req.body)) : "{}";
          const body = JSON.parse(bodyStr);
          return Response.json({ created: true, ...body });
        },
      },
      "/api/*": Response.json({ message: "Not found" }, { status: 404 }),
      "/blog/hello": Response.redirect("/blog/hello/world"),
      "/favicon.ico": file("./favicon.ico"),
    },
    fetch(_req: any) {
      return new Response("Not Found Fallback", { status: 404 });
    },
  });

  expect(server.port).toBe(port);
  expect(server.url).toBe(`http://127.0.0.1:${port}`);

  await new Promise((r) => setTimeout(r, 200));

  try {
    // 1. Static response
    const res1 = await fetch(`${server.url}/api/status`);
    expect(res1.status).toBe(200);
    expect(await res1.text()).toBe("OK");

    // 2. Dynamic route parameter
    const res2 = await fetch(`${server.url}/users/42`);
    expect(res2.status).toBe(200);
    expect(await res2.text()).toBe("Hello User 42!");

    // 3. Per-method GET
    const res3Get = await fetch(`${server.url}/api/posts`);
    expect(res3Get.status).toBe(200);
    expect(await res3Get.text()).toBe("List posts");

    // 4. Per-method POST
    const res3Post = await fetch(`${server.url}/api/posts`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ title: "New Post" }),
    });
    expect(res3Post.status).toBe(200);
    expect(await res3Post.json()).toEqual({ created: true, title: "New Post" });

    // 5. Wildcard route
    const res4 = await fetch(`${server.url}/api/unmatched`);
    expect(res4.status).toBe(404);
    expect(await res4.json()).toEqual({ message: "Not found" });

    // 6. Redirect
    const res5 = await fetch(`${server.url}/blog/hello`, { redirect: "manual" });
    expect(res5.status).toBe(302);
    expect(res5.headers.get("Location")).toBe("/blog/hello/world");
  } finally {
    server.stop();
  }
});

test("registerRoute registers custom route before serve", async () => {
  registerRoute("GET", "/registered/route", async () => new Response("Registered Handler"));

  const port = 3095;
  const server = Alloy.serve({ port, hostname: "127.0.0.1" });

  await new Promise((r) => setTimeout(r, 200));

  try {
    const res = await fetch(`${server.url}/registered/route`);
    expect(res.status).toBe(200);
    expect(await res.text()).toBe("Registered Handler");
  } finally {
    server.stop();
  }
});

test("App builder methods mount routes and launch Rocket server", async () => {
  const app = new App();
  app.get("/hello", async () => response("Hello from App!"));

  const port = 3094;
  const server = app.launch({ port, hostname: "127.0.0.1" });

  await new Promise((r) => setTimeout(r, 200));

  try {
    const res = await fetch(`${server.url}/hello`);
    expect(res.status).toBe(200);
    expect(await res.text()).toBe("Hello from App!");
  } finally {
    app.stop();
  }
});

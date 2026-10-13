import { expect, test } from "vite-plus/test";
import {
  serve,
  response,
  responseJson,
  responseRedirect,
  file,
  registerRoute,
  type RequestInput,
} from "../dist/index.js";

test("response helpers create valid response objects", () => {
  const resp = response("hello world", { status: 201, headers: { "X-Custom": "1" } });
  expect(resp.status).toBe(201);
  expect(resp.headers["X-Custom"]).toBe("1");
  expect(resp.body).toBeDefined();

  const jsonResp = responseJson({ message: "success" });
  expect(jsonResp.status).toBe(200);
  expect(jsonResp.headers["Content-Type"]).toBe("application/json");

  const redir = responseRedirect("/dashboard", 301);
  expect(redir.status).toBe(301);
  expect(redir.headers["Location"]).toBe("/dashboard");

  const staticFile = file("./assets/index.html");
  expect(staticFile.path).toBe("./assets/index.html");
});

test("serve allocates a random port when port is 0", async () => {
  const server = serve(
    {
      "/status": () => response("ok"),
    },
    null,
    { port: 0, hostname: "127.0.0.1" },
  );

  expect(server.port).toBeGreaterThan(0);
  expect(server.url).toBe(`http://127.0.0.1:${server.port}`);

  await new Promise((r) => setTimeout(r, 150));

  try {
    const res = await fetch(`${server.url}/status`);
    expect(res.status).toBe(200);
    expect(await res.text()).toBe("ok");
  } finally {
    server.stop();
  }
});

test("serve starts Rocket web server and handles routes, params, JSON, redirects, static files and fetch fallback", async () => {
  // Register a route using low-level registerRoute
  registerRoute("GET", "/registered", () => response("from registered route"));

  const server = serve(
    {
      "/ping": () => response("pong"),
      "/async-ping": async () => response("async pong"),
      "/json": () => responseJson({ name: "Alloy", ok: true }),
      "/users/:id": (req: RequestInput) => response(`User: ${req.params?.["id"]}`),
      "/redirect": () => responseRedirect("/ping"),
      "/posts": {
        GET: () => response("List posts"),
        POST: (_req: RequestInput) => responseJson({ created: true }),
      },
      "/file": file("./ReadMe.md"),
    },
    (req: RequestInput) => response(`Fallback for ${req.path}`, { status: 404 }),
    { port: 9870, hostname: "127.0.0.1" },
  );

  expect(server.port).toBe(9870);
  expect(server.url).toBe("http://127.0.0.1:9870");

  // Wait for server liftoff
  await new Promise((r) => setTimeout(r, 150));

  try {
    // 1. Basic route
    const resPing = await fetch("http://127.0.0.1:9870/ping");
    expect(resPing.status).toBe(200);
    expect(await resPing.text()).toBe("pong");

    // 2. Async route
    const resAsync = await fetch("http://127.0.0.1:9870/async-ping");
    expect(resAsync.status).toBe(200);
    expect(await resAsync.text()).toBe("async pong");

    // 3. JSON route
    const resJson = await fetch("http://127.0.0.1:9870/json");
    expect(resJson.status).toBe(200);
    expect(resJson.headers.get("content-type")).toContain("application/json");
    expect(await resJson.json()).toEqual({ name: "Alloy", ok: true });

    // 4. Path parameter route
    const resUser = await fetch("http://127.0.0.1:9870/users/42");
    expect(resUser.status).toBe(200);
    expect(await resUser.text()).toBe("User: 42");

    // 5. Redirect route
    const resRedir = await fetch("http://127.0.0.1:9870/redirect", { redirect: "manual" });
    expect(resRedir.status).toBe(302);
    expect(resRedir.headers.get("location")).toBe("/ping");

    // 6. Per-method routes
    const resPostsGet = await fetch("http://127.0.0.1:9870/posts");
    expect(resPostsGet.status).toBe(200);
    expect(await resPostsGet.text()).toBe("List posts");

    const resPostsPost = await fetch("http://127.0.0.1:9870/posts", { method: "POST" });
    expect(resPostsPost.status).toBe(200);
    expect(await resPostsPost.json()).toEqual({ created: true });

    // 7. Static file serving
    const resFile = await fetch("http://127.0.0.1:9870/file");
    expect(resFile.status).toBe(200);
    const fileContent = await resFile.text();
    expect(fileContent).toContain("# Server");

    // 8. Programmatically registered route
    const resRegistered = await fetch("http://127.0.0.1:9870/registered");
    expect(resRegistered.status).toBe(200);
    expect(await resRegistered.text()).toBe("from registered route");

    // 9. Fetch fallback for unmatched route
    const resFallback = await fetch("http://127.0.0.1:9870/non-existent-route");
    expect(resFallback.status).toBe(404);
    expect(await resFallback.text()).toBe("Fallback for /non-existent-route");
  } finally {
    server.stop();
  }
});

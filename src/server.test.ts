import { writeFileSync, unlinkSync, existsSync } from "node:fs";
import { join } from "node:path";
import { expect, test } from "vite-plus/test";
import {
  serve,
  Alloy,
  response,
  responseJson,
  responseRedirect,
  file,
  registerRoute,
  type RequestInput,
} from "../dist/index.js";

// Helper for HTTP requests
async function fetchServer(
  url: string,
  options?: RequestInit,
): Promise<{ status: number; headers: Headers; bodyText: string; json?: any }> {
  const res = await fetch(url, options);
  const bodyText = await res.text();
  let json: any;
  try {
    json = JSON.parse(bodyText);
  } catch {
    // ignore
  }
  return { status: res.status, headers: res.headers, bodyText, json };
}

test("Response constructor helpers produce correct structure", () => {
  const r1 = response("Hello World", { status: 200, headers: { "X-Test": "1" } });
  expect(r1.status).toBe(200);
  expect(r1.headers["X-Test"]).toBe("1");
  expect(r1.body).toEqual(Array.from(Buffer.from("Hello World")));

  const r2 = responseJson({ message: "Success" }, { status: 201 });
  expect(r2.status).toBe(201);
  expect(r2.headers["Content-Type"]).toBe("application/json");
  expect(r2.body).toBeDefined();
  expect(JSON.parse(Buffer.from(r2.body!).toString("utf-8"))).toEqual({ message: "Success" });

  const r3 = responseRedirect("http://example.com/login", 301);
  expect(r3.status).toBe(301);
  expect(r3.headers["Location"]).toBe("http://example.com/login");
});

test("file helper creates static file marker", () => {
  const sf = file("./static/test.txt");
  expect(sf.path).toBe("./static/test.txt");
});

test("Alloy.serve boots Bun.serve compatible server with object options", async () => {
  const tempFilePath = join(process.cwd(), "test-file.txt");
  writeFileSync(tempFilePath, "Static File Content", "utf-8");

  try {
    const server = Alloy.serve({
      port: 0, // dynamic port selection
      hostname: "127.0.0.1",
      routes: {
        "/api/status": response("OK"),
        "/users/:id": async (req: RequestInput) => response(`Hello User ${req.params["id"]}!`),
        "/api/posts": {
          GET: async () => response("List posts"),
          POST: async (req: RequestInput) => {
            const bodyStr = req.body ? Buffer.from(req.body).toString("utf-8") : "";
            const parsed = bodyStr ? JSON.parse(bodyStr) : {};
            return responseJson({ created: true, ...parsed });
          },
        },
        "/api/*": responseJson({ message: "Not found" }, { status: 404 }),
        "/blog/hello": responseRedirect("/blog/hello/world"),
        "/favicon.ico": file(tempFilePath),
        "/html-page": "<h1>HTML Page</h1>",
        "/html-import": { html: "<div>App Component</div>" },
      },
      async fetch(req: RequestInput) {
        return responseJson({ fallback: true, path: req.path }, { status: 404 });
      },
    });

    expect(server.port).toBeGreaterThan(0);
    expect(server.url).toBe(`http://127.0.0.1:${server.port}`);

    await new Promise((resolve) => setTimeout(resolve, 300));

    // Test static route
    const resStatus = await fetchServer(`${server.url}/api/status`);
    expect(resStatus.status).toBe(200);
    expect(resStatus.bodyText).toBe("OK");

    // Test dynamic route /users/:id
    const resUser = await fetchServer(`${server.url}/users/42`);
    expect(resUser.status).toBe(200);
    expect(resUser.bodyText).toBe("Hello User 42!");

    // Test per-HTTP method handlers
    const resGetPosts = await fetchServer(`${server.url}/api/posts`);
    expect(resGetPosts.bodyText).toBe("List posts");

    const resPostPosts = await fetchServer(`${server.url}/api/posts`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ title: "New Post" }),
    });
    expect(resPostPosts.json).toEqual({ created: true, title: "New Post" });

    // Test redirect
    const resRedir = await fetchServer(`${server.url}/blog/hello`, { redirect: "manual" });
    expect(resRedir.status).toBe(302);
    expect(resRedir.headers.get("location")).toBe("/blog/hello/world");

    // Test favicon.ico file
    const resFavicon = await fetchServer(`${server.url}/favicon.ico`);
    expect(resFavicon.status).toBe(200);
    expect(resFavicon.bodyText).toBe("Static File Content");

    // Test HTML string route
    const resHtml = await fetchServer(`${server.url}/html-page`);
    expect(resHtml.status).toBe(200);
    expect(resHtml.headers.get("content-type")).toContain("text/html");
    expect(resHtml.bodyText).toBe("<h1>HTML Page</h1>");

    // Test HTML import route object
    const resHtmlImport = await fetchServer(`${server.url}/html-import`);
    expect(resHtmlImport.status).toBe(200);
    expect(resHtmlImport.bodyText).toBe("<div>App Component</div>");

    // Test wildcard fallback for /api/*
    const resApiWildcard = await fetchServer(`${server.url}/api/unknown`);
    expect(resApiWildcard.status).toBe(404);
    expect(resApiWildcard.json).toEqual({ message: "Not found" });

    server.stop();
  } finally {
    if (existsSync(tempFilePath)) {
      unlinkSync(tempFilePath);
    }
  }
});

test("Environment variable port fallback works", async () => {
  process.env["PORT"] = "3888";
  try {
    const server = serve({
      routes: {
        "/ping": async () => response("pong"),
      },
      hostname: "127.0.0.1",
    });

    expect(server.port).toBe(3888);
    expect(server.url).toBe("http://127.0.0.1:3888");

    await new Promise((resolve) => setTimeout(resolve, 300));

    const res = await fetchServer(`${server.url}/ping`);
    expect(res.bodyText).toBe("pong");

    server.stop();
  } finally {
    delete process.env["PORT"];
  }
});

test("registerRoute registers individual routes before serve", async () => {
  const port = 3459;

  registerRoute("GET", "/registered", async () => response("Registered Handler"));

  const server = serve(
    {
      "/hello": async () => response("Hello!"),
    },
    undefined,
    { port, hostname: "127.0.0.1" },
  );

  await new Promise((resolve) => setTimeout(resolve, 300));

  const resRegistered = await fetchServer(`${server.url}/registered`);
  expect(resRegistered.status).toBe(200);
  expect(resRegistered.bodyText).toBe("Registered Handler");

  server.stop();
});

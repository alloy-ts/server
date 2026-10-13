import { writeFileSync, unlinkSync, existsSync } from "node:fs";
import { join } from "node:path";
import { expect, test } from "vite-plus/test";
import {
  serve,
  response,
  responseJson,
  responseRedirect,
  file,
  registerRoute,
  RequestInput,
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

test("serve boots Rocket HTTP server with routes table, method maps, params, and stops gracefully", async () => {
  const port = 3456;
  const tempFilePath = join(process.cwd(), "test-file.txt");
  writeFileSync(tempFilePath, "Static File Content", "utf-8");

  try {
    const server = serve(
      {
        "/hello": async () => response("Hello from Rocket!"),
        "/json": async () => responseJson({ status: "ok", count: 42 }),
        "/redirect": async () => responseRedirect("/hello"),
        "/users/:id": async (req: RequestInput) =>
          responseJson({ userId: req.params.id, query: req.query }),
        "/methods": {
          GET: async () => response("GET response"),
          POST: async (req: RequestInput) => {
            const bodyStr = req.body ? Buffer.from(req.body).toString("utf-8") : "";
            return responseJson({ received: bodyStr });
          },
        },
        "/static-file": file(tempFilePath),
      },
      async (req: RequestInput) =>
        responseJson({ fallback: true, path: req.path }, { status: 404 }),
      { port, hostname: "127.0.0.1" },
    );

    expect(server.url).toBe(`http://127.0.0.1:${port}`);
    expect(server.port).toBe(port);

    // Give the server a brief moment to liftoff
    await new Promise((resolve) => setTimeout(resolve, 300));

    // Test GET /hello
    const resHello = await fetchServer(`${server.url}/hello`);
    expect(resHello.status).toBe(200);
    expect(resHello.bodyText).toBe("Hello from Rocket!");

    // Test GET /json
    const resJson = await fetchServer(`${server.url}/json`);
    expect(resJson.status).toBe(200);
    expect(resJson.headers.get("content-type")).toContain("application/json");
    expect(resJson.json).toEqual({ status: "ok", count: 42 });

    // Test GET /redirect
    const resRedir = await fetchServer(`${server.url}/redirect`, { redirect: "manual" });
    expect(resRedir.status).toBe(302);
    expect(resRedir.headers.get("location")).toBe("/hello");

    // Test GET /users/123?page=2
    const resUser = await fetchServer(`${server.url}/users/123?page=2`);
    expect(resUser.status).toBe(200);
    expect(resUser.json.userId).toBe("123");
    expect(resUser.json.query).toBe("page=2");

    // Test GET /methods and POST /methods
    const resMethodGet = await fetchServer(`${server.url}/methods`);
    expect(resMethodGet.bodyText).toBe("GET response");

    const resMethodPost = await fetchServer(`${server.url}/methods`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ name: "Alloy" }),
    });
    expect(resMethodPost.json).toEqual({ received: '{"name":"Alloy"}' });

    // Test static file route
    const resFile = await fetchServer(`${server.url}/static-file`);
    expect(resFile.status).toBe(200);
    expect(resFile.bodyText).toBe("Static File Content");

    // Test fallback handler for unmatched route
    const resFallback = await fetchServer(`${server.url}/unmatched-path`);
    expect(resFallback.status).toBe(404);
    expect(resFallback.json).toEqual({ fallback: true, path: "/unmatched-path" });

    // Gracefully stop
    server.stop();
  } finally {
    if (existsSync(tempFilePath)) {
      unlinkSync(tempFilePath);
    }
  }
});

test("registerRoute registers individual routes before serve", async () => {
  const port = 3457;

  registerRoute("GET", "/registered", async () => response("Registered Handler"));

  const server = serve(
    {
      "/hello": () => response("Hello!"),
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

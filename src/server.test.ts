import { expect, test, describe, afterAll } from "vite-plus/test";
import {
  serve,
  response,
  responseJson,
  responseRedirect,
  file,
  registerRoute,
  Server,
  App,
  type RequestInput,
} from "../dist/index.js";

describe("Rocket server NAPI bindings", () => {
  describe("Response constructors / helpers", () => {
    test("response creates plain Response object", () => {
      const res = response("Hello World", { status: 201, headers: { "X-Test": "true" } });
      expect(res.status).toBe(201);
      expect(res.headers["X-Test"]).toBe("true");
      expect(res.body).toEqual(Array.from(Buffer.from("Hello World")));
    });

    test("response defaults status to 200", () => {
      const res = response("OK");
      expect(res.status).toBe(200);
      expect(res.headers).toEqual({});
      expect(res.body).toEqual(Array.from(Buffer.from("OK")));
    });

    test("responseJson stringifies data and sets content-type", () => {
      const data = { message: "success", code: 200 };
      const res = responseJson(data);

      expect(res.status).toBe(200);
      expect(res.headers["Content-Type"]).toBe("application/json");

      const bodyStr = Buffer.from(res.body ?? []).toString("utf-8");
      expect(JSON.parse(bodyStr)).toEqual(data);
    });

    test("responseRedirect creates 302 redirect response", () => {
      const res = responseRedirect("https://example.com/login", 301);
      expect(res.status).toBe(301);
      expect(res.headers["Location"]).toBe("https://example.com/login");
      expect(res.body).toBeUndefined();
    });

    test("file returns StaticFile marker object", () => {
      const sf = file("./favicon.ico");
      expect(sf.path).toBe("./favicon.ico");
    });
  });

  describe("HTTP server lifecycle and routing", () => {
    let server: Server | null = null;
    const PORT = 3099;

    afterAll(() => {
      if (server) {
        server.stop();
      }
    });

    test("serve starts HTTP server and responds to requests", async () => {
      server = serve(
        {
          "/api/status": response("OK"),
          "/users/:id": async (req: RequestInput) => {
            return response(`Hello ${req.params["id"]}`);
          },
          "/api/posts": {
            GET: async () => response("List posts"),
            POST: async () => responseJson({ created: true }),
          },
        },
        async (req: RequestInput) => {
          return response(`Not Found: ${req.path}`, { status: 404 });
        },
        { port: PORT, hostname: "127.0.0.1" },
      );

      expect(server).toBeInstanceOf(Server);
      expect(server.port).toBe(PORT);
      expect(server.url).toBe(`http://127.0.0.1:${PORT}`);

      // Allow server background thread time to launch
      await new Promise((resolve) => setTimeout(resolve, 300));

      // Test GET /api/status (static response)
      const res1 = await fetch(`${server.url}/api/status`);
      expect(res1.status).toBe(200);
      expect(await res1.text()).toBe("OK");

      // Test GET /users/123 (function route with params)
      const res2 = await fetch(`${server.url}/users/123`);
      expect(res2.status).toBe(200);
      expect(await res2.text()).toBe("Hello 123");

      // Test GET /api/posts (per-method route map)
      const res3 = await fetch(`${server.url}/api/posts`);
      expect(res3.status).toBe(200);
      expect(await res3.text()).toBe("List posts");

      // Test POST /api/posts (per-method route map JSON)
      const res4 = await fetch(`${server.url}/api/posts`, { method: "POST" });
      expect(res4.status).toBe(200);
      const json4 = await res4.json();
      expect(json4).toEqual({ created: true });

      // Test GET /nonexistent (fallback fetch handler)
      const res5 = await fetch(`${server.url}/nonexistent`);
      expect(res5.status).toBe(404);
      expect(await res5.text()).toBe("Not Found: /nonexistent");
    });

    test("server.stop shuts down the server gracefully", () => {
      expect(server).not.toBeNull();
      expect(() => server?.stop()).not.toThrow();
    });

    test("serve supports single options object format serve({ routes, fetch, port })", async () => {
      const s = serve({
        routes: {
          "/single-obj": response("SingleObjOK"),
        },
        async fetch(req: RequestInput) {
          return response(`Fallback ${req.path}`, { status: 404 });
        },
        port: 0,
        hostname: "127.0.0.1",
      });

      expect(s.port).toBeGreaterThan(0);
      expect(s.url).toBe(`http://127.0.0.1:${s.port}`);

      await new Promise((resolve) => setTimeout(resolve, 300));

      const res = await fetch(`${s.url}/single-obj`);
      expect(res.status).toBe(200);
      expect(await res.text()).toBe("SingleObjOK");

      s.stop();
    });

    test("serve with port 0 binds to a random port", async () => {
      const randomServer = serve(
        {
          "/health": response("OK"),
        },
        undefined,
        { port: 0, hostname: "127.0.0.1" },
      );

      expect(randomServer.port).toBeGreaterThan(0);
      expect(randomServer.url).toBe(`http://127.0.0.1:${randomServer.port}`);

      await new Promise((resolve) => setTimeout(resolve, 300));

      const res = await fetch(`${randomServer.url}/health`);
      expect(res.status).toBe(200);
      expect(await res.text()).toBe("OK");

      randomServer.stop();
    });

    test("serve supports unix option", () => {
      const unixServer = serve(
        {
          "/": response("OK"),
        },
        undefined,
        { unix: "/tmp/my-test-socket.sock" },
      );

      expect(unixServer.url).toBe("unix:///tmp/my-test-socket.sock");
      unixServer.stop();
    });
  });

  describe("App builder class", () => {
    test("App get, post, and launch register routes and handle HTTP requests", async () => {
      const app = new App();

      app.get("/app-hello", async () => response("Hello from App!"));
      app.post("/app-echo", async (req: RequestInput) => responseJson({ path: req.path }));

      const server = app.launch({ port: 0, hostname: "127.0.0.1" });

      expect(server.port).toBeGreaterThan(0);
      expect(server.url).toBe(`http://127.0.0.1:${server.port}`);

      await new Promise((resolve) => setTimeout(resolve, 300));

      const res1 = await fetch(`${server.url}/app-hello`);
      expect(res1.status).toBe(200);
      expect(await res1.text()).toBe("Hello from App!");

      const res2 = await fetch(`${server.url}/app-echo`, { method: "POST" });
      expect(res2.status).toBe(200);
      const json2 = await res2.json();
      expect(json2).toEqual({ path: "/app-echo" });

      app.stop();
    });
  });

  describe("registerRoute low-level API", () => {
    test("registerRoute registers route without throwing", () => {
      expect(() => {
        registerRoute("GET", "/low-level-test", async () => {
          return response("OK");
        });
      }).not.toThrow();
    });
  });
});

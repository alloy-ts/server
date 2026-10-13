import { expect, test } from "vite-plus/test";
import { serve, response, responseJson, responseRedirect, file, registerRoute } from "./main.ts";

test("response constructor creates Response object", () => {
  const res = response("Hello, Rocket!", { status: 200, headers: { "X-Test": "1" } });
  expect(res.status).toBe(200);
  expect(res.headers["X-Test"]).toBe("1");
  expect(res.body).toBeDefined();
});

test("responseJson constructs JSON response", () => {
  const res = responseJson({ message: "success", code: 200 });
  expect(res.status).toBe(200);
  expect(res.headers["Content-Type"]).toBe("application/json");
  expect(res.body).toBeDefined();
});

test("responseRedirect constructs redirect response", () => {
  const res = responseRedirect("/dashboard", 301);
  expect(res.status).toBe(301);
  expect(res.headers["Location"]).toBe("/dashboard");
});

test("file returns StaticFile object", () => {
  const sf = file("./public/index.html");
  expect(sf.path).toBe("./public/index.html");
});

test("serve initializes server and returns Server instance", () => {
  const server = serve(
    {
      "/health": responseJson({ status: "ok" }),
    },
    undefined,
    { port: 9090, hostname: "127.0.0.1" },
  );

  expect(server.url).toBe("http://127.0.0.1:9090");
  expect(server.port).toBe(9090);

  // Stop server
  server.stop();
});

test("registerRoute registers a route handler without throwing", () => {
  expect(() => {
    registerRoute("GET", "/registered", async (_req: any) => {
      return response("Registered");
    });
  }).not.toThrow();
});

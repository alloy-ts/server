import { expect, test } from "vite-plus/test";
import { App, file, response, responseJson, responseRedirect, serve } from "../dist/index.js";

test("response helper constructs Response payload", () => {
  const resp = response("Hello, world!", {
    status: 200,
    headers: { "X-Test": "Value" },
  });

  expect(resp.status).toBe(200);
  expect(resp.headers["X-Test"]).toBe("Value");
  expect(resp.body).toBeDefined();
});

test("responseJson helper constructs JSON Response payload", () => {
  const resp = responseJson({ key: "value" }, { status: 201 });

  expect(resp.status).toBe(201);
  expect(resp.headers["Content-Type"]).toBe("application/json");
  expect(resp.body).toBeDefined();
});

test("responseRedirect helper constructs redirect Response payload", () => {
  const resp = responseRedirect("/target", 301);

  expect(resp.status).toBe(301);
  expect(resp.headers["Location"]).toBe("/target");
});

test("file helper constructs StaticFile marker", () => {
  const marker = file("./test.png");
  expect(marker.path).toBe("./test.png");
});

test("serve creates and stops server handle", () => {
  const server = serve(
    {
      "/ping": response("pong"),
    },
    null,
    { port: 9876 },
  );

  expect(server.port).toBe(9876);
  expect(server.url).toBe("http://0.0.0.0:9876");

  expect(() => server.stop()).not.toThrow();
});

test("App instance configures routes and controls server lifecycle", () => {
  const app = new App();

  expect(() => {
    app.get("/", async () => response("OK"));
    app.post("/submit", async () => responseJson({ ok: true }));
  }).not.toThrow();

  expect(() => app.launch()).not.toThrow();
  expect(() => app.stop()).not.toThrow();
});

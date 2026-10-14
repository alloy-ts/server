const test = require("node:test");
const assert = require("node:assert/strict");

test.after(() => {
  setImmediate(() => process.exit(0));
});

void test("main() returns Hello, world!", async () => {
  const { main } = await import("./dist/main.mjs");
  assert.equal(main(), "Hello, world!");
});

void test("Rocket NAPI Config", async () => {
  const { Config } = await import("./dist/main.mjs");
  const cfg = new Config();
  assert.equal(typeof cfg.port, "number");
  cfg.port = 8080;
  assert.equal(cfg.port, 8080);
});

void test("Rocket NAPI Route", async () => {
  const { Route } = await import("./dist/main.mjs");
  const route = new Route("GET", "/test/<id>", () => {});
  assert.equal(route.method, "GET");
  assert.equal(route.uri, "/test/<id>");
  assert.equal(typeof route.rank, "number");
  route.name = "test_route";
  assert.equal(route.name, "test_route");
});

void test("Rocket NAPI RouteUri", async () => {
  const { RouteUri } = await import("./dist/main.mjs");
  const routeUri = new RouteUri("/base", "/foo/<bar>?a=1");
  assert.equal(routeUri.base, "/base");
  assert.equal(routeUri.unmountedOrigin, "/foo/<bar>?a=1");
  assert.equal(routeUri.origin, "/base/foo/<bar>?a=1");
  assert.equal(routeUri.path, "/base/foo/<bar>");
  assert.equal(routeUri.query, "a=1");
  assert.equal(routeUri.asStr, "/base/foo/<bar>?a=1");
});

void test("Rocket NAPI Handler", async () => {
  const { Handler, outcomeSuccess } = await import("./dist/main.mjs");
  const handler = new Handler(() => outcomeSuccess({ status: 200, body: Buffer.from("OK") }));
  assert.ok(handler);
});

void test("Rocket NAPI Codegen", async () => {
  const { codegen } = await import("./dist/main.mjs");

  const route = codegen.get("/users/:id", { name: "get_user" });
  assert.equal(route.method, "GET");
  assert.equal(route.path, "/users/:id");
  assert.equal(route.rocketPath, "/users/<id>");
  assert.equal(route.name, "get_user");

  const catcher = codegen.catch(404, "not_found");
  assert.equal(catcher.code, 404);
  assert.equal(catcher.name, "not_found");

  const uri = codegen.uri("/person/:name", { name: "Alice", age: "30" });
  assert.equal(uri.uri, "/person/Alice?age=30");

  const tsDecl = codegen.generateTsDeclarations([route]);
  assert.ok(tsDecl.includes("export interface RocketRoutes"));
  assert.ok(tsDecl.includes("GET /users/:id"));
});

void test("Alloy serve HTTP server", async () => {
  const { serve, Response } = await import("./dist/main.mjs");

  const server = serve({
    port: 0,
    routes: {
      "/api/status": new Response("OK"),
      "/users/:id": (req) => new Response(`Hello ${req.params.id}!`),
      "/api/posts": {
        GET: () => Response.json({ posts: ["Post 1"] }),
      },
    },
    fetch() {
      return new Response("Not Found", { status: 404 });
    },
  });

  assert.ok(server.port > 0);
  assert.ok(server.url.startsWith("http://"));

  const res1 = await fetch(`${server.url}/api/status`);
  assert.equal(res1.status, 200);
  assert.equal(await res1.text(), "OK");

  const res2 = await fetch(`${server.url}/users/42`);
  assert.equal(res2.status, 200);
  assert.equal(await res2.text(), "Hello 42!");

  const res3 = await fetch(`${server.url}/api/posts`);
  assert.equal(res3.status, 200);
  const data = await res3.json();
  assert.deepEqual(data, { posts: ["Post 1"] });

  server.stop();
});

const test = require("node:test");
const assert = require("node:assert");

let native;
try {
  native = require("./dist/alloy-server.linux-x64-gnu.node");
} catch {
  native = require("./dist/index.js");
}

void test("native module exports expected functions and classes", () => {
  assert.strictEqual(typeof native.serve, "function");
  assert.strictEqual(typeof native.response, "function");
  assert.strictEqual(typeof native.responseJson, "function");
  assert.strictEqual(typeof native.responseRedirect, "function");
  assert.strictEqual(typeof native.file, "function");
  assert.strictEqual(typeof native.parseRouteUri, "function");
  assert.strictEqual(typeof native.translatePattern, "function");
  assert.strictEqual(typeof native.generateTsUriBuilder, "function");
  assert.strictEqual(typeof native.parseCatcherStatus, "function");
  assert.strictEqual(typeof native.Config, "function");
  assert.strictEqual(typeof native.Rocket, "function");
});

void test("Response constructors work as expected", () => {
  const resp = native.response("hello world", { status: 200 });
  assert.strictEqual(resp.status, 200);

  const jsonResp = native.responseJson({ success: true });
  assert.strictEqual(jsonResp.status, 200);
  assert.strictEqual(jsonResp.headers["Content-Type"], "application/json");

  const redir = native.responseRedirect("/login", 302);
  assert.strictEqual(redir.status, 302);
  assert.strictEqual(redir.headers["Location"], "/login");
});

void test("Codegen helpers parse patterns and generate URI builders", () => {
  const pattern = native.translatePattern("/users/:id/*");
  assert.strictEqual(pattern, "/users/<id>/<_..>");

  const meta = native.parseRouteUri("/users/:id", "GET");
  assert.strictEqual(meta.method, "GET");
  assert.strictEqual(meta.rocketPath, "/users/<id>");
  assert.strictEqual(meta.params.length, 1);
  assert.strictEqual(meta.params[0].name, "id");

  const tsCode = native.generateTsUriBuilder([
    { name: "getUser", method: "GET", path: "/users/:id" },
  ]);
  assert.ok(tsCode.includes("getUser: (id: string | number) => `/users/${id}`"));

  assert.strictEqual(native.parseCatcherStatus("404"), 404);
  assert.strictEqual(native.parseCatcherStatus("default"), null);
});

void test("Config instance creation and getters/setters", () => {
  const config = new native.Config();
  assert.strictEqual(config.port, 8000);
  config.port = 9090;
  assert.strictEqual(config.port, 9090);
});

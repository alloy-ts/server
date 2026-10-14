const test = require("node:test");
const assert = require("node:assert");
const native = require("./dist/index.js");

void test("Response constructor and helpers", () => {
  const res = new native.Response("hello world", { status: 200 });
  assert.strictEqual(res.status, 200);

  const jsonRes = native.Response.json({ ok: true }, { status: 201 });
  assert.strictEqual(jsonRes.status, 201);

  const redirectRes = native.Response.redirect("https://rocket.rs", 302);
  assert.strictEqual(redirectRes.status, 302);
});

void test("file helper", () => {
  const sf = native.file("./package.json");
  assert.strictEqual(sf.path, "./package.json");
});

void test("Config default factory and getters", () => {
  const config = native.Config.debug_default();
  assert.strictEqual(typeof config.port, "number");
  assert.strictEqual(typeof config.address, "string");
});

void test("serve and stop", async () => {
  const server = native.serve(
    {
      "/ping": new native.Response("pong"),
    },
    null,
    { port: 0 },
  );

  assert.ok(server);
  assert.strictEqual(typeof server.port, "number");
  assert.strictEqual(typeof server.url, "string");

  assert.doesNotThrow(() => {
    server.stop();
  });
});

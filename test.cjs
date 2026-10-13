const test = require("node:test");
const assert = require("node:assert");

void test("Response constructor and helpers", async () => {
  const { Response, Config, file, serve } = await import("./dist/main.mjs");

  const res = new Response("hello world", { status: 200 });
  assert.strictEqual(res.status, 200);

  const jsonRes = Response.json({ ok: true }, { status: 201 });
  assert.strictEqual(jsonRes.status, 201);

  const redirectRes = Response.redirect("https://rocket.rs", 302);
  assert.strictEqual(redirectRes.status, 302);

  const sf = file("./package.json");
  assert.strictEqual(sf.path, "./package.json");

  if (Config) {
    const config = Config.debug_default();
    assert.strictEqual(typeof config.port, "number");
    assert.strictEqual(typeof config.address, "string");
  }

  const server = serve(
    {
      "/ping": new Response("pong"),
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

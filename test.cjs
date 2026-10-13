const test = require("node:test");
const assert = require("node:assert");
const native = require("./dist/index.js");

void test("CJS import of NAPI bindings", () => {
  assert.strictEqual(typeof native.Config, "function");
  assert.strictEqual(typeof native.serve, "function");
  assert.strictEqual(typeof native.response, "function");

  const cfg = native.Config.debugDefault();
  assert.strictEqual(cfg.port, 8000);
  assert.strictEqual(cfg.address, "127.0.0.1");
});

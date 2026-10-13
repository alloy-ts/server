const test = require("node:test");
const assert = require("node:assert");
const native = require("./dist/index.js");

void test("CJS export check", () => {
  assert.ok(native);
  assert.strictEqual(typeof native.serve, "function");
  assert.strictEqual(typeof native.buildUri, "function");
  assert.strictEqual(typeof native.Response, "function");
});

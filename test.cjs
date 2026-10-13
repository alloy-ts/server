const assert = require("node:assert");
const test = require("node:test");

void test("CJS loading dist/index.js", () => {
  const native = require("./dist/index.js");
  assert.strictEqual(typeof native.serve, "function");
  assert.strictEqual(typeof native.Response, "function");
  assert.strictEqual(typeof native.Config, "function");
  assert.strictEqual(typeof native.buildRouteUri, "function");
});

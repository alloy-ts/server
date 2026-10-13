const assert = require("node:assert");
const { Alloy, Bun, serve, response, responseJson, responseRedirect, file } = require("./dist/index.js");

assert.strictEqual(typeof serve, "function");
assert.strictEqual(typeof Alloy.serve, "function");
assert.strictEqual(typeof Bun.serve, "function");
assert.strictEqual(typeof file, "function");
assert.strictEqual(typeof response, "function");

const resp = response("ok");
assert.strictEqual(resp.status, 200);

const jsonResp = responseJson({ hello: "world" });
assert.strictEqual(jsonResp.status, 200);

console.log("CJS NAPI binding test passed!");

const assert = require("node:assert");
const { main, Config, Codegen, uri, responseJson } = require("./dist/main.mjs");

console.log("Running node --test test.cjs...");

assert.strictEqual(main(), "Hello, world!");

const cfg = new Config();
assert.ok(cfg.port > 0);

const meta = Codegen.parseRoute("POST", "/api/posts", { format: "json" });
assert.strictEqual(meta.method, "POST");

const builtUri = uri("/person/<name>?<age>", { name: "Alice", age: 25 });
assert.strictEqual(builtUri, "/person/Alice?age=25");

const resp = responseJson({ ok: true });
assert.strictEqual(resp.status, 200);

console.log("test.cjs passed successfully!");

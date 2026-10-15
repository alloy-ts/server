const assert = require("node:assert");
const { Config, Codegen, buildUri, responseJson, Rocket } = require("./dist/index.js");

console.log("Running node --test test.cjs...");

const cfg = new Config();
assert.ok(cfg.port > 0);

const meta = Codegen.parseRoute("POST", "/api/posts", "json");
assert.strictEqual(meta.method, "POST");

const uriStr = buildUri("/person/<name>?<age>", { name: "Alice", age: "25" });
assert.strictEqual(uriStr, "/person/Alice?age=25");

const resp = responseJson({ ok: true });
assert.strictEqual(resp.status, 200);

const rkt = Rocket.build();
assert.ok(rkt.config.port > 0);

console.log("test.cjs passed successfully!");

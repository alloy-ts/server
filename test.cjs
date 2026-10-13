const assert = require("assert");
const test = require("node:test");
const server = require("./dist/index.js");

void test("parseRocketRoutePattern in CJS", () => {
  const params = server.parseRocketRoutePattern("/users/<id>?<age>&<name..>");
  assert.strictEqual(params.length, 3);
  assert.strictEqual(params[0].name, "id");
  assert.strictEqual(params[0].isPath, true);
  assert.strictEqual(params[2].name, "name");
  assert.strictEqual(params[2].isOptional, true);
});

void test("translatePatternToAlloy in CJS", () => {
  assert.strictEqual(server.translatePatternToAlloy("/users/<id>/<rest..>"), "/users/:id/*");
});

void test("generateTypescriptClient in CJS", () => {
  const result = server.generateTypescriptClient([
    {
      method: "GET",
      path: "/users/<id>",
      name: "getUser",
    },
  ]);
  assert.ok(result.code.includes("export class ApiClient"));
});

void test("Config in CJS", () => {
  const cfg = server.Config.debugDefault();
  assert.strictEqual(typeof cfg.port, "number");
});

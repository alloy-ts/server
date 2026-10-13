const test = require("node:test");
const assert = require("node:assert/strict");
const { httpMethods, parseRoute, toRocketPath } = require("./dist/index.js");

void test("CJS native binding import", () => {
  const methods = httpMethods();
  assert.equal(methods.length, 9);
  assert.equal(toRocketPath("/users/:id"), "/users/<id>");
  const route = parseRoute({ method: "GET", path: "/users/:id" });
  assert.equal(route.key, "GET /users/:id");
});

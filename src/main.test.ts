import test from "node:test";
import assert from "node:assert/strict";
import { serve, Config, Route, RouteUri, Handler } from "../index.js";

test("Main entrypoint imports", () => {
  assert.equal(typeof serve, "function");
  assert.equal(typeof Config, "function");
  assert.equal(typeof Route, "function");
  assert.equal(typeof RouteUri, "function");
  assert.equal(typeof Handler, "function");
});

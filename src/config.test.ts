import test from "node:test";
import assert from "node:assert/strict";
import { Config } from "../index.js";

test("Config default instantiation", () => {
  const config = new Config();
  assert.equal(typeof config.port, "number");
  assert.equal(typeof config.address, "string");
  assert.equal(typeof config.workers, "number");
});

test("Config setters and getters", () => {
  const config = new Config();
  config.port = 8080;
  assert.equal(config.port, 8080);

  config.address = "127.0.0.1";
  assert.equal(config.address, "127.0.0.1");

  config.workers = 4;
  assert.equal(config.workers, 4);

  config.ident = "CustomApp";
  assert.equal(config.ident, "CustomApp");

  config.logLevel = "normal";
  assert.equal(config.logLevel, "normal");
});

test("Config factory methods", () => {
  const debug = Config.debugDefault();
  assert.ok(debug);
  assert.equal(typeof debug.port, "number");

  const release = Config.releaseDefault();
  assert.ok(release);
  assert.equal(typeof release.port, "number");

  const figment = Config.figment();
  assert.ok(figment);

  const params = Config.parameters();
  assert.ok(Array.isArray(params));
  assert.ok(params.includes("address"));
  assert.ok(params.includes("port"));
});

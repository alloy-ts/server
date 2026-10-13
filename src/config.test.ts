import { expect, test } from "vite-plus/test";
import { Config, DEBUG_PROFILE, RELEASE_PROFILE } from "../dist/index.js";

test("Config constants are exported", () => {
  expect(DEBUG_PROFILE).toBe("debug");
  expect(RELEASE_PROFILE).toBe("release");
});

test("Config constructor creates default debug config", () => {
  const config = new Config();
  expect(config.profile).toBe("debug");
  expect(config.port).toBe(8000);
  expect(config.address).toBe("127.0.0.1");
});

test("Config.debugDefault and releaseDefault factories work", () => {
  const debug = Config.debugDefault();
  expect(debug.profile).toBe("debug");

  const release = Config.releaseDefault();
  expect(release.profile).toBe("release");
});

test("Config setters update parameters", () => {
  const config = new Config();
  config.port = 9000;
  config.address = "0.0.0.0";
  config.workers = 8;

  expect(config.port).toBe(9000);
  expect(config.address).toBe("0.0.0.0");
  expect(config.workers).toBe(8);
});

test("Config.tryFrom applies overrides", () => {
  const config = Config.tryFrom({
    port: 8080,
    address: "127.0.0.1",
    logLevel: "normal",
  });

  expect(config.port).toBe(8080);
  expect(config.address).toBe("127.0.0.1");
  expect(config.logLevel).toBe("normal");
});

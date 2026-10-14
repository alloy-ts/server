import { expect, test } from "vite-plus/test";
import { Config, DEBUG_PROFILE, DEFAULT_PROFILE, RELEASE_PROFILE } from "./main.ts";

test("Config constants", () => {
  expect(DEBUG_PROFILE).toBe("debug");
  expect(RELEASE_PROFILE).toBe("release");
  expect(DEFAULT_PROFILE).toBe("debug");
});

test("Config default constructors", () => {
  const config = new Config();
  expect(config.port).toBe(8000);
  expect(config.address).toBe("127.0.0.1");

  const debugCfg = Config.debugDefault();
  expect(debugCfg.port).toBe(8000);

  const releaseCfg = Config.releaseDefault();
  expect(releaseCfg.port).toBe(8000);
});

test("Config getters and setters", () => {
  const config = new Config();
  config.port = 8080;
  expect(config.port).toBe(8080);

  config.address = "0.0.0.0";
  expect(config.address).toBe("0.0.0.0");

  config.logLevel = "critical";
  expect(config.logLevel).toBe("critical");
});

test("Config tryFrom override builder", () => {
  const config = Config.tryFrom({
    port: 9000,
    address: "127.0.0.1",
    logLevel: "debug",
  });
  expect(config.port).toBe(9000);
  expect(config.address).toBe("127.0.0.1");
});

test("Config parameters list", () => {
  const params = Config.parameters();
  expect(params).toContain("port");
  expect(params).toContain("address");
  expect(params).toContain("workers");
});

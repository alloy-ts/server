import { expect, test } from "vite-plus/test";
import { Config } from "./main.ts";

test("Config.debugDefault and Config.releaseDefault return default configurations", () => {
  const debugCfg = Config.debugDefault();
  expect(debugCfg.address).toBe("127.0.0.1");
  expect(debugCfg.port).toBe(8000);
  expect(debugCfg.logLevel).toBe("normal");

  const releaseCfg = Config.releaseDefault();
  expect(releaseCfg.address).toBe("127.0.0.1");
  expect(releaseCfg.port).toBe(8000);
});

test("Config getters and setters mutate configuration values", () => {
  const cfg = new Config();
  cfg.port = 9090;
  expect(cfg.port).toBe(9090);

  cfg.address = "0.0.0.0";
  expect(cfg.address).toBe("0.0.0.0");

  cfg.logLevel = "critical";
  expect(cfg.logLevel).toBe("critical");
});

test("Config.tryFrom applies partial overrides onto debug defaults", () => {
  const customCfg = Config.tryFrom({
    port: 8080,
    address: "127.0.0.1",
    logLevel: "debug",
    workers: 8,
  });
  expect(customCfg.port).toBe(8080);
  expect(customCfg.address).toBe("127.0.0.1");
  expect(customCfg.logLevel).toBe("debug");
  expect(customCfg.workers).toBe(8);
});

test("Config.parameters exposes all configuration parameter names", () => {
  const params = Config.parameters();
  expect(params).toContain("address");
  expect(params).toContain("port");
  expect(params).toContain("workers");
  expect(params).toContain("log_level");
});

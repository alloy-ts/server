import { expect, test } from "vite-plus/test";
import { Config } from "./main.ts";

test("Config static factories and defaults work correctly", () => {
  const debugCfg = Config.debugDefault();
  expect(debugCfg).toBeDefined();
  expect(debugCfg.address).toBe("127.0.0.1");
  expect(debugCfg.port).toBe(8000);

  const releaseCfg = Config.releaseDefault();
  expect(releaseCfg).toBeDefined();

  const defaultCfg = Config.defaultConfig();
  expect(defaultCfg).toBeDefined();

  const params = Config.parameters();
  expect(Array.isArray(params)).toBe(true);
  expect(params).toContain("address");
  expect(params).toContain("port");
});

test("Config tryFrom creates overridden config", () => {
  const cfg = Config.tryFrom({
    address: "0.0.0.0",
    port: 9090,
    workers: 8,
    logLevel: "debug",
  });

  expect(cfg.address).toBe("0.0.0.0");
  expect(cfg.port).toBe(9090);
  expect(cfg.workers).toBe(8);
  expect(cfg.logLevel).toBe("debug");
});

test("Config getters and setters update configuration fields", () => {
  const cfg = new Config();

  cfg.address = "192.168.1.1";
  expect(cfg.address).toBe("192.168.1.1");

  cfg.port = 8080;
  expect(cfg.port).toBe(8080);

  cfg.workers = 16;
  expect(cfg.workers).toBe(16);

  cfg.maxBlocking = 256;
  expect(cfg.maxBlocking).toBe(256);

  cfg.ident = "CustomServer";
  expect(cfg.ident).toBe("CustomServer");

  cfg.ipHeader = "X-Forwarded-For";
  expect(cfg.ipHeader).toBe("X-Forwarded-For");

  cfg.logLevel = "normal";
  expect(cfg.logLevel).toBe("normal");

  cfg.cliColors = false;
  expect(cfg.cliColors).toBe(false);

  cfg.keepAlive = 10;
  expect(cfg.keepAlive).toBe(10);

  cfg.shutdown = { grace: 3, mercy: 3, ctrlc: true };
  expect(cfg.shutdown.grace).toBe(3);

  cfg.limits = { limits: { form: "64KiB", json: "2MiB" } };
  expect(cfg.limits.limits["form"]).toBe("64KiB");
});

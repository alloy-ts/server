import { expect, test } from "vite-plus/test";
import { Config, DEBUG_PROFILE, RELEASE_PROFILE, DEFAULT_PROFILE, ADDRESS, PORT } from "./main.ts";

test("Config constants match Rocket parameters and profiles", () => {
  expect(DEBUG_PROFILE).toBe("debug");
  expect(RELEASE_PROFILE).toBe("release");
  expect(DEFAULT_PROFILE).toBe("debug");
  expect(ADDRESS).toBe("address");
  expect(PORT).toBe("port");

  const parameters = Config.parameters();
  expect(parameters).toContain("address");
  expect(parameters).toContain("port");
  expect(parameters).toContain("limits");
  expect(parameters).toContain("shutdown");
});

test("Config default factories initialize valid configurations", () => {
  const debugCfg = Config.debugDefault();
  expect(debugCfg.profile).toBe("debug");
  expect(debugCfg.port).toBeGreaterThan(0);

  const releaseCfg = Config.releaseDefault();
  expect(releaseCfg.profile).toBe("release");

  const defaultCfg = Config.defaultConfig();
  expect(defaultCfg.port).toBeGreaterThan(0);
});

test("Config getter and setter mutation and eager validation", () => {
  const cfg = new Config();

  cfg.port = 8000;
  expect(cfg.port).toBe(8000);

  cfg.workers = 16;
  expect(cfg.workers).toBe(16);

  cfg.maxBlocking = 512;
  expect(cfg.maxBlocking).toBe(512);

  cfg.keepAlive = 10;
  expect(cfg.keepAlive).toBe(10);

  cfg.logLevel = "normal";
  expect(cfg.logLevel).toBe("normal");

  cfg.cliColors = false;
  expect(cfg.cliColors).toBe(false);

  cfg.address = "127.0.0.1";
  expect(cfg.address).toBe("127.0.0.1");

  expect(() => {
    cfg.address = "invalid-ip";
  }).toThrow();

  cfg.limits = { limits: { form: "64KiB", json: "10MiB" } };
  expect(cfg.limits.limits["form"]).toBe("64KiB");

  cfg.shutdown = { ctrlc: true, grace: 10 };
  expect(cfg.shutdown.grace).toBe(10);
});

test("Config tryFrom partial overrides", () => {
  const cfg = Config.tryFrom({
    port: 9001,
    address: "0.0.0.0",
    workers: 8,
    logLevel: "debug",
    limits: { limits: { json: "2MiB" } },
  });

  expect(cfg.port).toBe(9001);
  expect(cfg.address).toBe("0.0.0.0");
  expect(cfg.workers).toBe(8);
  expect(cfg.logLevel).toBe("debug");
  expect(cfg.limits.limits["json"]).toBe("2MiB");
});

test("Config tlsEnabled query", () => {
  const cfg = new Config();
  expect(cfg.tlsEnabled()).toBe(false);
});

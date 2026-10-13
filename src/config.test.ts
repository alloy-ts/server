import { expect, test } from "vite-plus/test";
import {
  ADDRESS,
  Config,
  DEBUG_PROFILE,
  DEFAULT_PROFILE,
  IDENT,
  KEEP_ALIVE,
  LIMITS,
  LOG_LEVEL,
  MAX_BLOCKING,
  PORT,
  RELEASE_PROFILE,
  SHUTDOWN,
  TEMP_DIR,
  WORKERS,
} from "../dist/index.js";

test("Config constants match expected Rocket defaults", () => {
  expect(DEBUG_PROFILE).toBe("debug");
  expect(RELEASE_PROFILE).toBe("release");
  expect(DEFAULT_PROFILE).toBe("debug");
  expect(ADDRESS).toBe("address");
  expect(PORT).toBe("port");
  expect(WORKERS).toBe("workers");
  expect(MAX_BLOCKING).toBe("max_blocking");
  expect(KEEP_ALIVE).toBe("keep_alive");
  expect(IDENT).toBe("ident");
  expect(LIMITS).toBe("limits");
  expect(TEMP_DIR).toBe("temp_dir");
  expect(LOG_LEVEL).toBe("log_level");
  expect(SHUTDOWN).toBe("shutdown");
});

test("Config.debugDefault initializes debug configuration", () => {
  const cfg = Config.debugDefault();
  expect(cfg.port).toBe(8000);
  expect(cfg.address).toBe("127.0.0.1");
  expect(cfg.logLevel).toBe("normal");
  expect(cfg.profile).toBe("debug");
});

test("Config.releaseDefault initializes release configuration", () => {
  const cfg = Config.releaseDefault();
  expect(cfg.port).toBe(8000);
  expect(cfg.profile).toBe("release");
  expect(cfg.logLevel).toBe("critical");
});

test("Config.tryFrom applies partial overrides", () => {
  const cfg = Config.tryFrom({
    port: 9000,
    address: "0.0.0.0",
    logLevel: "debug",
    keepAlive: 10,
  });

  expect(cfg.port).toBe(9000);
  expect(cfg.address).toBe("0.0.0.0");
  expect(cfg.logLevel).toBe("debug");
  expect(cfg.keepAlive).toBe(10);
});

test("Config getters and setters mutate properties", () => {
  const cfg = new Config();
  cfg.port = 4000;
  expect(cfg.port).toBe(4000);

  cfg.address = "192.168.1.1";
  expect(cfg.address).toBe("192.168.1.1");

  cfg.logLevel = "off";
  expect(cfg.logLevel).toBe("off");
});

test("Config parameters returns list of configuration parameters", () => {
  const params = Config.parameters();
  expect(params).toContain("address");
  expect(params).toContain("port");
  expect(params).toContain("workers");
  expect(params).toContain("log_level");
});

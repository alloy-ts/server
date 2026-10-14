import { expect, test } from "vite-plus/test";
import { Config, DEBUG_PROFILE, RELEASE_PROFILE, PORT, ADDRESS } from "../dist/index.js";

test("Config constants match expected names", () => {
  expect(DEBUG_PROFILE).toBe("debug");
  expect(RELEASE_PROFILE).toBe("release");
  expect(PORT).toBe("port");
  expect(ADDRESS).toBe("address");
});

test("Config.debugDefault creates default debug config", () => {
  const cfg = Config.debugDefault();
  expect(cfg.port).toBeDefined();
  expect(cfg.address).toBeDefined();
  expect(cfg.logLevel).toBe("normal");
  expect(cfg.cliColors).toBe(true);
});

test("Config.releaseDefault creates default release config", () => {
  const cfg = Config.releaseDefault();
  expect(cfg.port).toBeDefined();
  expect(cfg.logLevel).toBe("critical");
});

test("Config getters and setters mutate properties", () => {
  const cfg = new Config();
  cfg.port = 9090;
  expect(cfg.port).toBe(9090);

  cfg.address = "127.0.0.1";
  expect(cfg.address).toBe("127.0.0.1");

  cfg.logLevel = "debug";
  expect(cfg.logLevel).toBe("debug");

  cfg.cliColors = false;
  expect(cfg.cliColors).toBe(false);
});

test("Config.tryFrom builds config with overrides", () => {
  const cfg = Config.tryFrom({
    port: 8080,
    address: "0.0.0.0",
    logLevel: "debug",
    limits: {
      limits: {
        json: "1MiB",
      },
    },
    shutdown: {
      ctrlc: true,
      grace: 5,
    },
  });

  expect(cfg.port).toBe(8080);
  expect(cfg.address).toBe("0.0.0.0");
  expect(cfg.logLevel).toBe("debug");
  expect(cfg.shutdown.grace).toBe(5);
});

test("Config.parameters returns parameter list", () => {
  const params = Config.parameters();
  expect(params).toContain("address");
  expect(params).toContain("port");
  expect(params).toContain("log_level");
});

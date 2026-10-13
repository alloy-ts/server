import { expect, test } from "vite-plus/test";
import { Config, type ConfigInit, ADDRESS, PORT, WORKERS } from "../dist/index.js";

test("Config class static factory methods construct Config instances", () => {
  const debugCfg = Config.debugDefault();
  expect(debugCfg).toBeDefined();
  expect(typeof debugCfg.port).toBe("number");
  expect(typeof debugCfg.address).toBe("string");

  const releaseCfg = Config.releaseDefault();
  expect(releaseCfg).toBeDefined();

  const defaultCfg = Config.defaultConfig();
  expect(defaultCfg).toBeDefined();
});

test("Config getters and setters update configuration properties", () => {
  const config = new Config();

  config.port = 9090;
  expect(config.port).toBe(9090);

  config.address = "127.0.0.1";
  expect(config.address).toBe("127.0.0.1");

  config.logLevel = "debug";
  expect(config.logLevel).toBe("debug");

  config.workers = 8;
  expect(config.workers).toBe(8);

  config.maxBlocking = 256;
  expect(config.maxBlocking).toBe(256);

  config.keepAlive = 10;
  expect(config.keepAlive).toBe(10);

  config.cliColors = false;
  expect(config.cliColors).toBe(false);
});

test("Config.tryFrom applies partial overrides", () => {
  const init: ConfigInit = {
    port: 8080,
    address: "0.0.0.0",
    logLevel: "critical",
    workers: 4,
    limits: {
      limits: {
        json: "10MiB",
        form: "64KiB",
      },
    },
    shutdown: {
      ctrlc: true,
      grace: 5,
      mercy: 5,
    },
  };

  const config = Config.tryFrom(init);
  expect(config.port).toBe(8080);
  expect(config.address).toBe("0.0.0.0");
  expect(config.logLevel).toBe("critical");
  expect(config.workers).toBe(4);

  const limits = config.limits;
  expect(limits.limits["json"]).toBe("10MiB");
  expect(limits.limits["form"]).toBe("64KiB");

  const shutdown = config.shutdown;
  expect(shutdown.ctrlc).toBe(true);
  expect(shutdown.grace).toBe(5);
  expect(shutdown.mercy).toBe(5);
});

test("Config parameter constants are defined", () => {
  expect(ADDRESS).toBe("address");
  expect(PORT).toBe("port");
  expect(WORKERS).toBe("workers");

  const params = Config.parameters();
  expect(Array.isArray(params)).toBe(true);
  expect(params).toContain("address");
  expect(params).toContain("port");
  expect(params).toContain("limits");
});

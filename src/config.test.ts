import { expect, test } from "vite-plus/test";
import { Config } from "../index.js";

test("config.ts - Config constructors and default profiles", () => {
  const debug = Config.debugDefault();
  expect(debug.port).toBe(8000);
  expect(debug.address).toBe("127.0.0.1");
  expect(debug.logLevel).toBe("normal");

  const release = Config.releaseDefault();
  expect(release.port).toBe(8000);

  const defaultConfig = Config.defaultConfig();
  expect(defaultConfig.port).toBe(8000);
});

test("config.ts - Config parameters list", () => {
  const params = Config.parameters();
  expect(params).toContain("address");
  expect(params).toContain("port");
  expect(params).toContain("workers");
  expect(params).toContain("limits");
  expect(params).toContain("shutdown");
});

test("config.ts - Config getters and setters", () => {
  const config = new Config();

  config.port = 8080;
  expect(config.port).toBe(8080);

  config.address = "0.0.0.0";
  expect(config.address).toBe("0.0.0.0");

  config.workers = 8;
  expect(config.workers).toBe(8);

  config.maxBlocking = 256;
  expect(config.maxBlocking).toBe(256);

  config.keepAlive = 10;
  expect(config.keepAlive).toBe(10);

  config.logLevel = "critical";
  expect(config.logLevel).toBe("critical");

  config.cliColors = false;
  expect(config.cliColors).toBe(false);

  config.tempDir = "/custom/tmp";
  expect(config.tempDir).toBe("/custom/tmp");
});

test("config.ts - Config.tryFrom with partial overrides", () => {
  const config = Config.tryFrom({
    port: 3000,
    address: "127.0.0.1",
    logLevel: "debug",
    workers: 4,
    shutdown: {
      grace: 10,
      mercy: 10,
      ctrlc: true,
    },
  });

  expect(config.port).toBe(3000);
  expect(config.address).toBe("127.0.0.1");
  expect(config.logLevel).toBe("debug");
  expect(config.workers).toBe(4);
  expect(config.shutdown.grace).toBe(10);
});

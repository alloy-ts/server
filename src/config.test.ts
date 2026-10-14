import { expect, test } from "vite-plus/test";
import { Config } from "../dist/index.js";

test("Config.debugDefault() creates configuration with debug profile defaults", () => {
  const config = Config.debugDefault();
  expect(config.profile).toBe("debug");
  expect(config.address).toBe("127.0.0.1");
  expect(config.port).toBe(8000);
  expect(config.logLevel).toBe("normal");
});

test("Config.releaseDefault() creates configuration with release profile defaults", () => {
  const config = Config.releaseDefault();
  expect(config.profile).toBe("release");
  expect(config.logLevel).toBe("critical");
});

test("Config.parameters() lists supported configuration parameter names", () => {
  const params = Config.parameters();
  expect(params).toContain("address");
  expect(params).toContain("port");
  expect(params).toContain("workers");
  expect(params).toContain("limits");
  expect(params).toContain("shutdown");
});

test("Config getters and setters mutate configuration properties", () => {
  const config = new Config();
  config.port = 9090;
  expect(config.port).toBe(9090);

  config.address = "0.0.0.0";
  expect(config.address).toBe("0.0.0.0");

  config.logLevel = "debug";
  expect(config.logLevel).toBe("debug");

  config.cliColors = false;
  expect(config.cliColors).toBe(false);
});

test("Config.tryFrom() applies partial configuration overrides", () => {
  const config = Config.tryFrom({
    port: 4000,
    address: "127.0.0.1",
    workers: 8,
    logLevel: "critical",
    limits: {
      limits: {
        json: "1MiB",
      },
    },
    shutdown: {
      ctrlc: true,
      grace: 10,
    },
  });

  expect(config.port).toBe(4000);
  expect(config.address).toBe("127.0.0.1");
  expect(config.workers).toBe(8);
  expect(config.logLevel).toBe("critical");
  expect(config.shutdown.grace).toBe(10);
});

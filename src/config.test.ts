import { expect, test } from "vite-plus/test";
import { Config, DEBUG_PROFILE, RELEASE_PROFILE, PORT, ADDRESS } from "../dist/index.js";

test("Config associated constants and static parameters", () => {
  expect(DEBUG_PROFILE).toBe("debug");
  expect(RELEASE_PROFILE).toBe("release");
  expect(PORT).toBe("port");
  expect(ADDRESS).toBe("address");

  const params = Config.parameters();
  expect(params).toContain("address");
  expect(params).toContain("port");
  expect(params).toContain("workers");
  expect(params).toContain("limits");
  expect(params).toContain("shutdown");
});

test("Config initializes with debug defaults", () => {
  const config = Config.debugDefault();
  expect(config.profile).toBe("debug");
  expect(config.port).toBe(8000);
  expect(config.address).toBe("127.0.0.1");
  expect(config.workers).toBeGreaterThan(0);
  expect(config.logLevel).toBe("normal");
  expect(config.cliColors).toBe(true);
});

test("Config supports release default constructor", () => {
  const config = Config.releaseDefault();
  expect(config.profile).toBe("release");
  expect(config.logLevel).toBe("critical");
});

test("Config getters and setters allow modifying properties", () => {
  const config = new Config();
  config.port = 9090;
  expect(config.port).toBe(9090);

  config.address = "0.0.0.0";
  expect(config.address).toBe("0.0.0.0");

  config.logLevel = "debug";
  expect(config.logLevel).toBe("debug");

  config.ident = "CustomApp";
  expect(config.ident).toBe("CustomApp");

  config.keepAlive = 10;
  expect(config.keepAlive).toBe(10);
});

test("Config tryFrom applies partial overrides", () => {
  const config = Config.tryFrom({
    port: 8080,
    address: "127.0.0.1",
    logLevel: "critical",
    shutdown: {
      grace: 10,
      mercy: 10,
    },
  });

  expect(config.port).toBe(8080);
  expect(config.address).toBe("127.0.0.1");
  expect(config.logLevel).toBe("critical");
  expect(config.shutdown.grace).toBe(10);
  expect(config.shutdown.mercy).toBe(10);
});

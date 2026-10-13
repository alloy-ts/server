import { expect, test } from "vite-plus/test";
import { Config } from "./main.ts";

test("Config initializes with debug defaults", () => {
  const config = Config.debugDefault();
  expect(config.port).toBe(8000);
  expect(config.address).toBe("127.0.0.1");
  expect(config.ident).toBe("Rocket");
  expect(config.ipHeader).toBe("X-Real-IP");
  expect(config.cliColors).toBe(true);
});

test("Config supports mutating properties via setters", () => {
  const config = new Config();
  config.port = 9090;
  config.address = "0.0.0.0";
  config.logLevel = "debug";

  expect(config.port).toBe(9090);
  expect(config.address).toBe("0.0.0.0");
  expect(config.logLevel).toBe("debug");
});

test("Config.tryFrom applies overrides", () => {
  const config = Config.tryFrom({
    port: 4000,
    address: "127.0.0.1",
    workers: 8,
    cliColors: false,
  });

  expect(config.port).toBe(4000);
  expect(config.workers).toBe(8);
  expect(config.cliColors).toBe(false);
});

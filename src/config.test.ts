import { expect, test } from "vite-plus/test";
import { Config } from "../dist/index.js";

test("Config.debug_default() returns debug defaults", () => {
  const config = Config.debug_default();
  expect(config.address).toBe("127.0.0.1");
  expect(config.port).toBe(8000);
  expect(config.logLevel).toBe("normal");
});

test("Config.release_default() returns release defaults", () => {
  const config = Config.release_default();
  expect(config.address).toBe("127.0.0.1");
  expect(config.port).toBe(8000);
});

test("Config.try_from overrides properties", () => {
  const config = Config.try_from({
    port: 9090,
    address: "0.0.0.0",
    logLevel: "debug",
  });
  expect(config.port).toBe(9090);
  expect(config.address).toBe("0.0.0.0");
  expect(config.logLevel).toBe("debug");
});

test("Config getters and setters mutate properties", () => {
  const config = new Config();
  config.port = 3000;
  expect(config.port).toBe(3000);

  config.address = "127.0.0.1";
  expect(config.address).toBe("127.0.0.1");
});

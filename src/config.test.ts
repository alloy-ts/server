import { expect, test } from "vite-plus/test";
import { Config } from "./main.ts";

test("Config.debug_default() returns debug defaults", () => {
  const config = Config.debug_default();
  expect(config.address).toBe("127.0.0.1");
  expect(typeof config.port).toBe("number");
  expect(typeof config.workers).toBe("number");
  expect(typeof config.maxBlocking).toBe("number");
  expect(config.ident).toBe("Rocket");
  expect(config.logLevel).toBe("normal");
});

test("Config.release_default() returns release defaults", () => {
  const config = Config.release_default();
  expect(config.address).toBe("127.0.0.1");
  expect(typeof config.port).toBe("number");
});

test("Config.try_from applies overrides", () => {
  const config = Config.try_from({
    port: 8080,
    address: "0.0.0.0",
    workers: 8,
    logLevel: "debug",
  });

  expect(config.port).toBe(8080);
  expect(config.address).toBe("0.0.0.0");
  expect(config.workers).toBe(8);
  expect(config.logLevel).toBe("debug");
});

test("Config getters and setters mutate properties", () => {
  const config = Config.debug_default();
  config.port = 9000;
  expect(config.port).toBe(9000);

  config.logLevel = "critical";
  expect(config.logLevel).toBe("critical");
});

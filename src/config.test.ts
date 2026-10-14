import { expect, test } from "vite-plus/test";
import { Config, DEBUG_PROFILE, DEFAULT_PROFILE, RELEASE_PROFILE } from "../index.js";

test("Config constants and defaults", () => {
  expect(DEBUG_PROFILE).toBe("debug");
  expect(RELEASE_PROFILE).toBe("release");
  expect(DEFAULT_PROFILE).toBe("debug");

  const cfg = new Config();
  expect(cfg.profile).toBe(DEBUG_PROFILE);
  expect(cfg.port).toBe(8000);

  const releaseCfg = Config.releaseDefault();
  expect(releaseCfg.profile).toBe(RELEASE_PROFILE);
});

test("Config mutation and tryFrom builder", () => {
  const cfg = Config.from({
    port: 9090,
    address: "127.0.0.1",
    workers: 4,
    logLevel: "normal",
  });

  expect(cfg.port).toBe(9090);
  expect(cfg.address).toBe("127.0.0.1");
  expect(cfg.workers).toBe(4);
  expect(cfg.logLevel).toBe("normal");

  cfg.port = 9091;
  expect(cfg.port).toBe(9091);
});

import { expect, test } from "vite-plus/test";
import { Config } from "./main.ts";

test("Config constructor and debug_default", () => {
  if (!Config) return;
  const cfg = new Config();
  expect(cfg.port).toBeGreaterThan(0);
  expect(cfg.address).toBeDefined();

  const debugCfg = Config.debug_default();
  expect(debugCfg.profile).toBe("debug");
  expect(debugCfg.port).toBe(8000);
});

test("Config release_default and default_config", () => {
  if (!Config) return;
  const relCfg = Config.release_default();
  expect(relCfg.profile).toBe("release");

  const defCfg = Config.default_config();
  expect(defCfg.address).toBeDefined();
});

test("Config parameters list", () => {
  if (!Config) return;
  const params = Config.parameters();
  expect(params).toContain("address");
  expect(params).toContain("port");
  expect(params).toContain("workers");
  expect(params).toContain("keep_alive");
  expect(params).toContain("limits");
  expect(params).toContain("shutdown");
});

test("Config getters and setters", () => {
  if (!Config) return;
  const cfg = new Config();
  cfg.port = 9090;
  expect(cfg.port).toBe(9090);

  cfg.workers = 8;
  expect(cfg.workers).toBe(8);

  cfg.keep_alive = 10;
  expect(cfg.keep_alive).toBe(10);

  cfg.cli_colors = false;
  expect(cfg.cli_colors).toBe(false);
});

test("Config try_from and from", () => {
  if (!Config) return;
  const cfg = Config.try_from({
    port: 8080,
    address: "127.0.0.1",
    workers: 4,
  });
  expect(cfg.port).toBe(8080);
  expect(cfg.address).toBe("127.0.0.1");
  expect(cfg.workers).toBe(4);

  const cfgFrom = Config.from({
    port: 3000,
  });
  expect(cfgFrom.port).toBe(3000);
});

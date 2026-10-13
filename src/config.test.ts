import { expect, test } from "vite-plus/test";
import {
  ADDRESS,
  CLI_COLORS,
  Config,
  DEBUG_PROFILE,
  DEFAULT_PROFILE,
  IDENT,
  IP_HEADER,
  KEEP_ALIVE,
  LIMITS,
  LOG_LEVEL,
  MAX_BLOCKING,
  PORT,
  RELEASE_PROFILE,
  SECRET_KEY,
  SHUTDOWN,
  TEMP_DIR,
  TLS,
  WORKERS,
} from "./main.ts";

test("Config parameter name constants", () => {
  expect(DEBUG_PROFILE).toBe("debug");
  expect(RELEASE_PROFILE).toBe("release");
  expect(DEFAULT_PROFILE).toBe("debug");

  expect(ADDRESS).toBe("address");
  expect(PORT).toBe("port");
  expect(WORKERS).toBe("workers");
  expect(MAX_BLOCKING).toBe("max_blocking");
  expect(KEEP_ALIVE).toBe("keep_alive");
  expect(IDENT).toBe("ident");
  expect(IP_HEADER).toBe("ip_header");
  expect(LIMITS).toBe("limits");
  expect(TLS).toBe("tls");
  expect(SECRET_KEY).toBe("secret_key");
  expect(TEMP_DIR).toBe("temp_dir");
  expect(LOG_LEVEL).toBe("log_level");
  expect(SHUTDOWN).toBe("shutdown");
  expect(CLI_COLORS).toBe("cli_colors");
});

test("Config static factories", () => {
  const debugCfg = Config.debugDefault();
  expect(debugCfg.profile).toBe("debug");

  const releaseCfg = Config.releaseDefault();
  expect(releaseCfg.profile).toBe("release");

  const defaultCfg = Config.defaultConfig();
  expect(defaultCfg).toBeDefined();

  const figmentCfg = Config.figment();
  expect(figmentCfg).toBeDefined();
});

test("Config getters and setters", () => {
  const cfg = new Config();

  cfg.profile = "custom";
  expect(cfg.profile).toBe("custom");

  cfg.address = "127.0.0.1";
  expect(cfg.address).toBe("127.0.0.1");

  cfg.port = 8080;
  expect(cfg.port).toBe(8080);

  cfg.workers = 8;
  expect(cfg.workers).toBe(8);

  cfg.maxBlocking = 256;
  expect(cfg.maxBlocking).toBe(256);

  cfg.ident = "CustomApp";
  expect(cfg.ident).toBe("CustomApp");

  cfg.ipHeader = "X-Forwarded-For";
  expect(cfg.ipHeader).toBe("X-Forwarded-For");

  cfg.limits = { limits: { json: "2MiB", form: "64KiB" } };
  expect(cfg.limits).toEqual({ limits: { json: "2MiB", form: "64KiB" } });

  cfg.tempDir = "/tmp/custom";
  expect(cfg.tempDir).toBe("/tmp/custom");

  cfg.keepAlive = 10;
  expect(cfg.keepAlive).toBe(10);

  cfg.shutdown = { ctrlc: true, grace: 3, mercy: 3 };
  expect(cfg.shutdown.grace).toBe(3);

  cfg.logLevel = "debug";
  expect(cfg.logLevel).toBe("debug");

  cfg.cliColors = false;
  expect(cfg.cliColors).toBe(false);
});

test("Config.tryFrom and Config.from with overrides", () => {
  const init = {
    port: 9000,
    address: "0.0.0.0",
    logLevel: "normal",
    limits: { limits: { form: "128KiB" } },
  };

  const cfg1 = Config.tryFrom(init);
  expect(cfg1.port).toBe(9000);
  expect(cfg1.address).toBe("0.0.0.0");
  expect(cfg1.limits).toEqual({ limits: { form: "128KiB" } });

  const cfg2 = Config.from(init);
  expect(cfg2.port).toBe(9000);
});

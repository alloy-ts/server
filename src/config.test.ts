import { expect, test } from "vite-plus/test";
import {
  Config,
  DEBUG_PROFILE,
  RELEASE_PROFILE,
  DEFAULT_PROFILE,
  ADDRESS,
  PORT,
  WORKERS,
  MAX_BLOCKING,
  KEEP_ALIVE,
  IDENT,
  IP_HEADER,
  LIMITS,
  TEMP_DIR,
  LOG_LEVEL,
  SHUTDOWN,
  CLI_COLORS,
} from "../dist/index.js";

test("Config constants match expected Rocket parameters and profiles", () => {
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
  expect(TEMP_DIR).toBe("temp_dir");
  expect(LOG_LEVEL).toBe("log_level");
  expect(SHUTDOWN).toBe("shutdown");
  expect(CLI_COLORS).toBe("cli_colors");

  const params = Config.parameters();
  expect(params).toContain(ADDRESS);
  expect(params).toContain(PORT);
  expect(params).toContain(WORKERS);
  expect(params).toContain(LOG_LEVEL);
});

test("Config debugDefault and default_config return debug defaults", () => {
  const cfg = Config.debugDefault();
  expect(cfg.profile).toBe("debug");
  expect(cfg.address).toBe("127.0.0.1");
  expect(cfg.port).toBe(8000);
  expect(cfg.ident).toBe("Rocket");
  expect(cfg.ipHeader).toBe("X-Real-IP");
  expect(cfg.logLevel).toBe("normal");
  expect(cfg.cliColors).toBe(true);
});

test("Config releaseDefault sets critical log_level and release profile", () => {
  const cfg = Config.releaseDefault();
  expect(cfg.profile).toBe("release");
  expect(cfg.logLevel).toBe("critical");
});

test("Config constructor and property getters/setters work properly", () => {
  const cfg = new Config();
  cfg.port = 9090;
  expect(cfg.port).toBe(9090);

  cfg.address = "0.0.0.0";
  expect(cfg.address).toBe("0.0.0.0");

  cfg.workers = 8;
  expect(cfg.workers).toBe(8);

  cfg.logLevel = "debug";
  expect(cfg.logLevel).toBe("debug");

  cfg.cliColors = false;
  expect(cfg.cliColors).toBe(false);

  cfg.ident = "MyCustomRocket";
  expect(cfg.ident).toBe("MyCustomRocket");
});

test("Config tryFrom and from build from partial overrides", () => {
  const cfg = Config.tryFrom({
    port: 8080,
    address: "127.0.0.1",
    workers: 16,
    logLevel: "critical",
    shutdown: {
      grace: 10,
      ctrlc: false,
    },
  });

  expect(cfg.port).toBe(8080);
  expect(cfg.address).toBe("127.0.0.1");
  expect(cfg.workers).toBe(16);
  expect(cfg.logLevel).toBe("critical");
  expect(cfg.shutdown.grace).toBe(10);
  expect(cfg.shutdown.ctrlc).toBe(false);

  const cfgFrom = Config.from({
    port: 9000,
    identNone: true,
  });
  expect(cfgFrom.port).toBe(9000);
  expect(cfgFrom.ident).toBeNull();
});

test("Config.figment reads active figment configuration", () => {
  const cfg = Config.figment();
  expect(cfg).toBeInstanceOf(Config);
  expect(cfg.port).toBeGreaterThan(0);
});

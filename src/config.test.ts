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
  TLS,
  SECRET_KEY,
  TEMP_DIR,
  LOG_LEVEL,
  SHUTDOWN,
  CLI_COLORS,
} from "../dist/index.js";

test("Config exported constants match Rocket parameter keys and profiles", () => {
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

  const params = Config.parameters();
  expect(params).toContain("port");
  expect(params).toContain("address");
  expect(params).toContain("workers");
  expect(params).toContain("limits");
});

test("Config static factory methods create valid Config instances", () => {
  const debugCfg = Config.debugDefault();
  expect(debugCfg.profile).toBe("debug");
  expect(debugCfg.port).toBe(8000);
  expect(debugCfg.address).toBe("127.0.0.1");

  const releaseCfg = Config.releaseDefault();
  expect(releaseCfg.profile).toBe("release");

  const defaultCfg = Config.defaultConfig();
  expect(defaultCfg.port).toBe(8000);

  const figmentCfg = Config.figment();
  expect(figmentCfg.port).toBeGreaterThan(0);
});

test("Config.tryFrom and Config.from allow partial overrides", () => {
  const custom = Config.tryFrom({
    port: 9000,
    address: "0.0.0.0",
    logLevel: "debug",
    cliColors: false,
    workers: 8,
    maxBlocking: 256,
    keepAlive: 10,
    tempDir: "/custom_tmp",
    ident: "CustomServer",
    ipHeader: "X-Forwarded-For",
    limits: {
      limits: {
        json: "10MiB",
        form: "64KiB",
      },
    },
    shutdown: {
      ctrlc: true,
      signals: ["term", "hup"],
      grace: 3,
      mercy: 5,
      force: true,
    },
  });

  expect(custom.port).toBe(9000);
  expect(custom.address).toBe("0.0.0.0");
  expect(custom.logLevel).toBe("debug");
  expect(custom.cliColors).toBe(false);
  expect(custom.workers).toBe(8);
  expect(custom.maxBlocking).toBe(256);
  expect(custom.keepAlive).toBe(10);
  expect(custom.tempDir).toBe("/custom_tmp");
  expect(custom.ident).toBe("CustomServer");
  expect(custom.ipHeader).toBe("X-Forwarded-For");

  const limits = custom.limits;
  expect(limits.limits["json"]).toBe("10MiB");
  expect(limits.limits["form"]).toBe("64KiB");

  const shutdown = custom.shutdown;
  expect(shutdown.ctrlc).toBe(true);
  expect(shutdown.grace).toBe(3);
  expect(shutdown.mercy).toBe(5);
  expect(shutdown.force).toBe(true);

  const fromCustom = Config.from({ port: 7000 });
  expect(fromCustom.port).toBe(7000);
});

test("Config getters and setters operate correctly", () => {
  const config = new Config();

  config.port = 8888;
  expect(config.port).toBe(8888);

  config.address = "192.168.1.1";
  expect(config.address).toBe("192.168.1.1");

  config.logLevel = "critical";
  expect(config.logLevel).toBe("critical");

  config.profile = "staging";
  expect(config.profile).toBe("staging");

  config.workers = 16;
  expect(config.workers).toBe(16);

  config.maxBlocking = 1024;
  expect(config.maxBlocking).toBe(1024);

  config.ident = "TestIdent";
  expect(config.ident).toBe("TestIdent");

  config.ipHeader = "X-Client-IP";
  expect(config.ipHeader).toBe("X-Client-IP");

  config.tempDir = "/tmp/test";
  expect(config.tempDir).toBe("/tmp/test");

  config.keepAlive = 15;
  expect(config.keepAlive).toBe(15);

  config.cliColors = true;
  expect(config.cliColors).toBe(true);
});

test("Config setters validate invalid inputs", () => {
  const config = new Config();

  expect(() => {
    config.address = "invalid_ip";
  }).toThrow();

  expect(() => {
    config.logLevel = "unknown_level";
  }).toThrow();

  expect(() => {
    config.shutdown = {
      ctrlc: true,
      signals: ["invalid_signal"],
      grace: 1,
      mercy: 1,
      force: false,
    };
  }).toThrow();

  expect(() => {
    config.limits = {
      limits: {
        form: "invalid_byte_unit",
      },
    };
  }).toThrow();
});

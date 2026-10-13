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
} from "./config.ts";

test("Config associated constants and parameter names", () => {
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
  expect(params).toContain("address");
  expect(params).toContain("port");
  expect(params).toContain("workers");
  expect(params).toContain("max_blocking");
  expect(params).toContain("keep_alive");
  expect(params).toContain("ident");
  expect(params).toContain("ip_header");
  expect(params).toContain("limits");
  expect(params).toContain("tls");
  expect(params).toContain("secret_key");
  expect(params).toContain("temp_dir");
  expect(params).toContain("log_level");
  expect(params).toContain("shutdown");
  expect(params).toContain("cli_colors");
});

test("Config debugDefault produces Rocket debug configuration defaults", () => {
  const cfg = Config.debugDefault();
  expect(cfg.profile).toBe("debug");
  expect(cfg.address).toBe("127.0.0.1");
  expect(cfg.port).toBe(8000);
  expect(cfg.ident).toBe("Rocket");
  expect(cfg.ipHeader).toBe("X-Real-IP");
  expect(cfg.keepAlive).toBe(5);
  expect(cfg.logLevel).toBe("normal");
  expect(cfg.cliColors).toBe(true);
  expect(typeof cfg.workers).toBe("number");
  expect(cfg.maxBlocking).toBe(512);
});

test("Config constructor creates debug defaults", () => {
  const cfg = new Config();
  expect(cfg.profile).toBe("debug");
  expect(cfg.address).toBe("127.0.0.1");
  expect(cfg.port).toBe(8000);
});

test("Config releaseDefault produces release defaults", () => {
  const cfg = Config.releaseDefault();
  expect(cfg.profile).toBe("release");
  expect(cfg.logLevel).toBe("critical");
});

test("Config defaultConfig returns active default config", () => {
  const cfg = Config.defaultConfig();
  expect(cfg.port).toBe(8000);
  expect(cfg.address).toBe("127.0.0.1");
});

test("Config figment extracts configuration from default figment provider", () => {
  const cfg = Config.figment();
  expect(typeof cfg.port).toBe("number");
  expect(typeof cfg.address).toBe("string");
});

test("Config tryFrom and from apply partial overrides", () => {
  const cfg = Config.tryFrom({
    profile: "staging",
    address: "0.0.0.0",
    port: 8080,
    workers: 4,
    maxBlocking: 256,
    ident: "MyApp/1.0",
    ipHeader: "X-Forwarded-For",
    tempDir: "/tmp/app",
    keepAlive: 15,
    logLevel: "debug",
    cliColors: false,
    limits: {
      limits: {
        json: "2MiB",
        form: "64KiB",
      },
    },
    shutdown: {
      ctrlc: false,
      grace: 3,
      mercy: 3,
      force: true,
    },
  });

  expect(cfg.profile).toBe("staging");
  expect(cfg.address).toBe("0.0.0.0");
  expect(cfg.port).toBe(8080);
  expect(cfg.workers).toBe(4);
  expect(cfg.maxBlocking).toBe(256);
  expect(cfg.ident).toBe("MyApp/1.0");
  expect(cfg.ipHeader).toBe("X-Forwarded-For");
  expect(cfg.tempDir).toBe("/tmp/app");
  expect(cfg.keepAlive).toBe(15);
  expect(cfg.logLevel).toBe("debug");
  expect(cfg.cliColors).toBe(false);
  expect(cfg.shutdown.ctrlc).toBe(false);
  expect(cfg.shutdown.grace).toBe(3);

  const cfgFrom = Config.from({
    identNone: true,
    ipHeaderNone: true,
  });
  expect(cfgFrom.ident).toBeNull();
  expect(cfgFrom.ipHeader).toBeNull();
});

test("Config getters and setters update fields correctly", () => {
  const cfg = new Config();

  cfg.profile = "production";
  expect(cfg.profile).toBe("production");

  cfg.address = "10.0.0.1";
  expect(cfg.address).toBe("10.0.0.1");
  expect(() => {
    cfg.address = "not-an-ip";
  }).toThrow();

  cfg.port = 9090;
  expect(cfg.port).toBe(9090);

  cfg.workers = 12;
  expect(cfg.workers).toBe(12);

  cfg.maxBlocking = 100;
  expect(cfg.maxBlocking).toBe(100);

  cfg.ident = "CustomServer";
  expect(cfg.ident).toBe("CustomServer");
  cfg.ident = null;
  expect(cfg.ident).toBeNull();

  cfg.ipHeader = "X-Client-IP";
  expect(cfg.ipHeader).toBe("X-Client-IP");
  cfg.ipHeader = null;
  expect(cfg.ipHeader).toBeNull();

  cfg.keepAlive = 30;
  expect(cfg.keepAlive).toBe(30);

  cfg.tempDir = "/var/tmp";
  expect(cfg.tempDir).toBe("/var/tmp");

  cfg.logLevel = "off";
  expect(cfg.logLevel).toBe("off");
  expect(() => {
    cfg.logLevel = "invalid_level";
  }).toThrow();

  cfg.cliColors = false;
  expect(cfg.cliColors).toBe(false);

  cfg.limits = { limits: { json: "10MiB" } };
  expect(cfg.limits.limits["json"]).toBe("10MiB");

  cfg.shutdown = { ctrlc: true, grace: 10 };
  expect(cfg.shutdown.ctrlc).toBe(true);
  expect(cfg.shutdown.grace).toBe(10);
});

test("Config tlsEnabled and mtlsEnabled return boolean status", () => {
  const cfg = new Config();
  expect(typeof cfg.tlsEnabled()).toBe("boolean");
  expect(typeof cfg.mtlsEnabled()).toBe("boolean");
});

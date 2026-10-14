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
} from "../dist/index.js";

test("Config default constructor and profile", () => {
  const config = new Config();
  expect(config.profile).toBe("debug");
  expect(config.port).toBe(8000);
  expect(config.address).toBe("127.0.0.1");
  expect(config.workers).toBeGreaterThan(0);
  expect(config.maxBlocking).toBe(512);
  expect(config.keepAlive).toBe(5);
  expect(config.ident).toBe("Rocket");
  expect(config.ipHeader).toBe("X-Real-IP");
  expect(config.logLevel).toBe("normal");
  expect(config.cliColors).toBe(true);
});

test("Config static profiles and factories", () => {
  expect(DEBUG_PROFILE).toBe("debug");
  expect(RELEASE_PROFILE).toBe("release");
  expect(DEFAULT_PROFILE).toBe("debug");

  const debugConfig = Config.debugDefault();
  expect(debugConfig.profile).toBe("debug");

  const releaseConfig = Config.releaseDefault();
  expect(releaseConfig.profile).toBe("release");

  const defaultConfig = Config.defaultConfig();
  expect(defaultConfig).toBeDefined();

  const figmentConfig = Config.figment();
  expect(figmentConfig).toBeDefined();
});

test("Config tryFrom and from overrides", () => {
  const custom = Config.tryFrom({
    port: 3000,
    address: "0.0.0.0",
    ident: "CustomServer",
    logLevel: "debug",
    cliColors: false,
    keepAlive: 10,
    workers: 8,
  });

  expect(custom.port).toBe(3000);
  expect(custom.address).toBe("0.0.0.0");
  expect(custom.ident).toBe("CustomServer");
  expect(custom.logLevel).toBe("debug");
  expect(custom.cliColors).toBe(false);
  expect(custom.keepAlive).toBe(10);
  expect(custom.workers).toBe(8);

  const fromConfig = Config.from({ port: 9090 });
  expect(fromConfig.port).toBe(9090);
});

test("Config getters, setters, and validation", () => {
  const config = new Config();

  config.port = 4000;
  expect(config.port).toBe(4000);

  config.address = "127.0.0.1";
  expect(config.address).toBe("127.0.0.1");

  config.profile = "staging";
  expect(config.profile).toBe("staging");

  config.ident = "TestApp";
  expect(config.ident).toBe("TestApp");

  config.ipHeader = "X-Forwarded-For";
  expect(config.ipHeader).toBe("X-Forwarded-For");

  config.tempDir = "/tmp/rocket";
  expect(config.tempDir).toBe("/tmp/rocket");

  config.keepAlive = 15;
  expect(config.keepAlive).toBe(15);

  config.workers = 16;
  expect(config.workers).toBe(16);

  config.maxBlocking = 256;
  expect(config.maxBlocking).toBe(256);

  config.limits = { limits: { form: "64KiB", json: "2MiB" } };
  expect(config.limits.limits["form"]).toBe("64KiB");

  config.shutdown = { ctrlc: false, grace: 10, mercy: 10, force: true };
  expect(config.shutdown.ctrlc).toBe(false);
  expect(config.shutdown.grace).toBe(10);
});

test("Config parameters list", () => {
  const params = Config.parameters();
  expect(params).toContain(ADDRESS);
  expect(params).toContain(PORT);
  expect(params).toContain(WORKERS);
  expect(params).toContain(MAX_BLOCKING);
  expect(params).toContain(KEEP_ALIVE);
  expect(params).toContain(IDENT);
  expect(params).toContain(IP_HEADER);
  expect(params).toContain(LIMITS);
  expect(params).toContain(TLS);
  expect(params).toContain(SECRET_KEY);
  expect(params).toContain(TEMP_DIR);
  expect(params).toContain(LOG_LEVEL);
  expect(params).toContain(SHUTDOWN);
  expect(params).toContain(CLI_COLORS);
});

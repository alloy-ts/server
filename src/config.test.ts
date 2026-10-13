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

test("Config constants match Rocket parameter names and profiles", () => {
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
  expect(params).toContain("log_level");
});

test("Config debugDefault and releaseDefault create expected defaults", () => {
  const debug = Config.debugDefault();
  expect(debug.profile).toBe("debug");
  expect(debug.port).toBe(8000);
  expect(debug.address).toBe("127.0.0.1");
  expect(debug.logLevel).toBe("normal");

  const release = Config.releaseDefault();
  expect(release.profile).toBe("release");
  expect(release.logLevel).toBe("critical");
});

test("Config getters and setters mutate properties correctly", () => {
  const cfg = new Config();

  cfg.port = 9090;
  expect(cfg.port).toBe(9090);

  cfg.address = "0.0.0.0";
  expect(cfg.address).toBe("0.0.0.0");

  cfg.logLevel = "debug";
  expect(cfg.logLevel).toBe("debug");

  cfg.ident = "CustomRocketServer";
  expect(cfg.ident).toBe("CustomRocketServer");

  cfg.cliColors = false;
  expect(cfg.cliColors).toBe(false);

  cfg.keepAlive = 10;
  expect(cfg.keepAlive).toBe(10);
});

test("Config throws on invalid setting value", () => {
  const cfg = new Config();

  expect(() => {
    cfg.address = "invalid-ip";
  }).toThrow();

  expect(() => {
    cfg.logLevel = "invalid-level";
  }).toThrow();
});

test("Config.tryFrom applies initial overrides", () => {
  const cfg = Config.tryFrom({
    port: 3000,
    address: "127.0.0.1",
    logLevel: "critical",
    ident: "AlloyServer",
    limits: {
      limits: {
        json: "10MiB",
        form: "64kB",
      },
    },
    shutdown: {
      ctrlc: true,
      grace: 3,
      mercy: 3,
    },
  });

  expect(cfg.port).toBe(3000);
  expect(cfg.address).toBe("127.0.0.1");
  expect(cfg.logLevel).toBe("critical");
  expect(cfg.ident).toBe("AlloyServer");
  expect(cfg.shutdown.grace).toBe(3);
  expect(cfg.shutdown.mercy).toBe(3);
});

test("Config.figment reads active provider config without crashing", () => {
  const cfg = Config.figment();
  expect(cfg).toBeDefined();
  expect(typeof cfg.port).toBe("number");
  expect(typeof cfg.address).toBe("string");
});

import { describe, expect, test } from "vite-plus/test";
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

describe("Rocket Config NAPI Bindings", () => {
  describe("Constants", () => {
    test("exports expected configuration parameter names and profiles", () => {
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
  });

  describe("Config Factories & Static Methods", () => {
    test("Config.debugDefault() creates a default debug config", () => {
      const cfg = Config.debugDefault();
      expect(cfg.profile).toBe("debug");
      expect(cfg.port).toBe(8000);
      expect(cfg.address).toBe("127.0.0.1");
      expect(cfg.cliColors).toBe(true);
      expect(cfg.logLevel).toBe("normal");
    });

    test("Config.releaseDefault() creates a release config", () => {
      const cfg = Config.releaseDefault();
      expect(cfg.profile).toBe("release");
      expect(cfg.port).toBe(8000);
      expect(cfg.logLevel).toBe("critical");
    });

    test("Config.tryFrom() applies partial overrides", () => {
      const cfg = Config.tryFrom({
        port: 9090,
        address: "0.0.0.0",
        logLevel: "debug",
      });

      expect(cfg.port).toBe(9090);
      expect(cfg.address).toBe("0.0.0.0");
      expect(cfg.logLevel).toBe("debug");
    });

    test("Config.parameters() lists all parameter names", () => {
      const params = Config.parameters();
      expect(params).toContain("address");
      expect(params).toContain("port");
      expect(params).toContain("workers");
      expect(params).toContain("log_level");
    });
  });

  describe("Getters and Setters", () => {
    test("allows reading and mutating scalar and object fields", () => {
      const cfg = new Config();

      cfg.port = 4000;
      expect(cfg.port).toBe(4000);

      cfg.address = "192.168.1.1";
      expect(cfg.address).toBe("192.168.1.1");

      cfg.logLevel = "debug";
      expect(cfg.logLevel).toBe("debug");

      cfg.cliColors = false;
      expect(cfg.cliColors).toBe(false);

      cfg.keepAlive = 10;
      expect(cfg.keepAlive).toBe(10);

      cfg.shutdown = { ctrlc: false, grace: 2, mercy: 3 };
      expect(cfg.shutdown.ctrlc).toBe(false);
      expect(cfg.shutdown.grace).toBe(2);
    });

    test("validates address and log_level input", () => {
      const cfg = new Config();
      expect(() => {
        cfg.address = "not-an-ip";
      }).toThrow();

      expect(() => {
        cfg.logLevel = "invalid-level";
      }).toThrow();
    });
  });
});

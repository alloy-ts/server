import { expect, test, describe } from "vite-plus/test";
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

describe("Rocket config NAPI bindings", () => {
  test("associated parameter and profile constants", () => {
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

  describe("static constructors and parameter listing", () => {
    test("debugDefault creates config with debug profile defaults", () => {
      const cfg = Config.debugDefault();
      expect(cfg.profile).toBe("debug");
      expect(cfg.port).toBe(8000);
      expect(cfg.address).toBe("127.0.0.1");
      expect(cfg.ident).toBe("Rocket");
      expect(cfg.cliColors).toBe(true);
    });

    test("releaseDefault creates config with release profile defaults", () => {
      const cfg = Config.releaseDefault();
      expect(cfg.profile).toBe("release");
      expect(cfg.port).toBe(8000);
    });

    test("defaultConfig creates default configuration", () => {
      const cfg = Config.defaultConfig();
      expect(cfg).toBeInstanceOf(Config);
    });

    test("figment extracts config from default figment provider", () => {
      const cfg = Config.figment();
      expect(cfg).toBeInstanceOf(Config);
      expect(cfg.address).toBeDefined();
    });

    test("tryFrom / from create config from partial init overrides", () => {
      const cfg = Config.tryFrom({
        port: 9000,
        address: "0.0.0.0",
        logLevel: "critical",
        workers: 8,
      });

      expect(cfg.port).toBe(9000);
      expect(cfg.address).toBe("0.0.0.0");
      expect(cfg.logLevel).toBe("critical");
      expect(cfg.workers).toBe(8);

      const cfg2 = Config.from({
        port: 9001,
        identNone: true,
      });

      expect(cfg2.port).toBe(9001);
      expect(cfg2.ident).toBeNull();
    });

    test("parameters returns array of parameter names", () => {
      const params = Config.parameters();
      expect(params).toContain("address");
      expect(params).toContain("port");
      expect(params).toContain("workers");
      expect(params).toContain("log_level");
      expect(params.length).toBe(14);
    });
  });

  describe("getters and setters", () => {
    test("mutates and inspects configuration properties", () => {
      const cfg = Config.debugDefault();

      cfg.profile = "custom";
      expect(cfg.profile).toBe("custom");

      cfg.address = "192.168.1.1";
      expect(cfg.address).toBe("192.168.1.1");

      cfg.port = 8080;
      expect(cfg.port).toBe(8080);

      cfg.workers = 16;
      expect(cfg.workers).toBe(16);

      cfg.maxBlocking = 256;
      expect(cfg.maxBlocking).toBe(256);

      cfg.ident = "MyRocketApp";
      expect(cfg.ident).toBe("MyRocketApp");

      cfg.ipHeader = "X-Forwarded-For";
      expect(cfg.ipHeader).toBe("X-Forwarded-For");

      cfg.tempDir = "/var/tmp";
      expect(cfg.tempDir).toBe("/var/tmp");

      cfg.keepAlive = 10;
      expect(cfg.keepAlive).toBe(10);

      cfg.cliColors = false;
      expect(cfg.cliColors).toBe(false);

      cfg.logLevel = "debug";
      expect(cfg.logLevel).toBe("debug");
    });

    test("sets and gets limits", () => {
      const cfg = Config.debugDefault();
      cfg.limits = {
        limits: {
          json: "10MiB",
          form: "64KiB",
        },
      };

      const lim = cfg.limits;
      expect(lim.limits["json"]).toBe("10MiB");
      expect(lim.limits["form"]).toBe("64KiB");
    });

    test("sets and gets shutdown settings", () => {
      const cfg = Config.debugDefault();
      cfg.shutdown = {
        ctrlc: false,
        grace: 10,
        mercy: 5,
        signals: ["term", "int"],
      };

      const sh = cfg.shutdown;
      expect(sh.ctrlc).toBe(false);
      expect(sh.grace).toBe(10);
      expect(sh.mercy).toBe(5);
      expect(sh.signals).toContain("term");
      expect(sh.signals).toContain("int");
    });

    test("eagerly validates property setters", () => {
      const cfg = Config.debugDefault();

      expect(() => {
        cfg.address = "not-an-ip";
      }).toThrow();

      expect(() => {
        cfg.logLevel = "invalid-log-level";
      }).toThrow();

      expect(() => {
        cfg.limits = {
          limits: {
            json: "not-a-valid-byte-size",
          },
        };
      }).toThrow();
    });
  });
});

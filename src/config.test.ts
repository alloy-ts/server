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
} from "./index.ts";

describe("Rocket Config NAPI Bindings", () => {
  test("exports profile and parameter constants", () => {
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

  test("Config.parameters() lists all parameter names", () => {
    const params = Config.parameters();
    expect(params).toContain("address");
    expect(params).toContain("port");
    expect(params).toContain("workers");
    expect(params).toContain("log_level");
    expect(params.length).toBeGreaterThanOrEqual(14);
  });

  describe("Constructors and Factory Methods", () => {
    test("Config constructor creates debug default instance", () => {
      const config = new Config();
      expect(config.profile).toBe("debug");
      expect(config.address).toBe("127.0.0.1");
      expect(config.port).toBe(8000);
      expect(config.logLevel).toBe("normal");
      expect(config.cliColors).toBe(true);
    });

    test("Config.debugDefault() creates debug profile defaults", () => {
      const config = Config.debugDefault();
      expect(config.profile).toBe("debug");
      expect(config.address).toBe("127.0.0.1");
      expect(config.port).toBe(8000);
      expect(config.logLevel).toBe("normal");
    });

    test("Config.releaseDefault() creates release profile defaults", () => {
      const config = Config.releaseDefault();
      expect(config.profile).toBe("release");
      expect(config.address).toBe("127.0.0.1");
      expect(config.port).toBe(8000);
      expect(config.logLevel).toBe("critical");
    });

    test("Config.defaultConfig() creates active default instance", () => {
      const config = Config.defaultConfig();
      expect(config.port).toBe(8000);
    });

    test("Config.figment() loads configuration from figment provider", () => {
      const config = Config.figment();
      expect(config.address).toBeDefined();
      expect(config.port).toBeGreaterThan(0);
    });

    test("Config.tryFrom() applies partial overrides", () => {
      const config = Config.tryFrom({
        port: 9001,
        address: "0.0.0.0",
        logLevel: "debug",
        cliColors: false,
      });

      expect(config.port).toBe(9001);
      expect(config.address).toBe("0.0.0.0");
      expect(config.logLevel).toBe("debug");
      expect(config.cliColors).toBe(false);
    });

    test("Config.from() applies overrides directly", () => {
      const config = Config.from({
        port: 8080,
        address: "127.0.0.1",
      });

      expect(config.port).toBe(8080);
      expect(config.address).toBe("127.0.0.1");
    });

    test("Config.tryFrom() throws error on invalid address or signal", () => {
      expect(() => {
        Config.tryFrom({ address: "invalid-ip" });
      }).toThrow();

      expect(() => {
        Config.tryFrom({
          shutdown: { signals: ["invalid_signal"] },
        });
      }).toThrow();
    });
  });

  describe("Getters and Setters", () => {
    test("mutates profile, address, and port", () => {
      const config = new Config();

      config.profile = "custom";
      expect(config.profile).toBe("custom");

      config.address = "0.0.0.0";
      expect(config.address).toBe("0.0.0.0");

      config.port = 4000;
      expect(config.port).toBe(4000);
    });

    test("validates address when setting", () => {
      const config = new Config();
      expect(() => {
        config.address = "not.an.ip";
      }).toThrow();
    });

    test("mutates workers and maxBlocking", () => {
      const config = new Config();

      config.workers = 16;
      expect(config.workers).toBe(16);

      config.maxBlocking = 256;
      expect(config.maxBlocking).toBe(256);
    });

    test("mutates ident and ipHeader", () => {
      const config = new Config();

      config.ident = "CustomApp";
      expect(config.ident).toBe("CustomApp");

      config.ident = null;
      expect(config.ident).toBeNull();

      config.ipHeader = "X-Forwarded-For";
      expect(config.ipHeader).toBe("X-Forwarded-For");

      config.ipHeader = null;
      expect(config.ipHeader).toBeNull();
    });

    test("mutates limits", () => {
      const config = new Config();

      config.limits = {
        limits: {
          form: "64KiB",
          json: "10MiB",
        },
      };

      expect(config.limits.limits).toEqual({
        form: "64KiB",
        json: "10MiB",
      });
    });

    test("mutates tempDir and keepAlive", () => {
      const config = new Config();

      config.tempDir = "/tmp/app";
      expect(config.tempDir).toBe("/tmp/app");

      config.keepAlive = 10;
      expect(config.keepAlive).toBe(10);
    });

    test("mutates shutdown options", () => {
      const config = new Config();

      config.shutdown = {
        ctrlc: true,
        signals: ["term", "hup"],
        grace: 10,
        mercy: 5,
        force: false,
      };

      const sd = config.shutdown;
      expect(sd.ctrlc).toBe(true);
      expect(sd.signals).toEqual(["term", "hup"]);
      expect(sd.grace).toBe(10);
      expect(sd.mercy).toBe(5);
      expect(sd.force).toBe(false);
    });

    test("mutates logLevel and cliColors", () => {
      const config = new Config();

      config.logLevel = "debug";
      expect(config.logLevel).toBe("debug");

      config.cliColors = false;
      expect(config.cliColors).toBe(false);
    });

    test("validates logLevel when setting", () => {
      const config = new Config();
      expect(() => {
        config.logLevel = "invalid_level";
      }).toThrow();
    });
  });
});

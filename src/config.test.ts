import { describe, expect, it } from "vite-plus/test";
import { Config } from "./main.ts";

describe("Config NAPI bindings", () => {
  it("new Config() creates instance with debug defaults", () => {
    const cfg = new Config();
    expect(cfg.address).toBe("127.0.0.1");
    expect(cfg.port).toBe(8000);
    expect(cfg.ident).toBe("Rocket");
    expect(cfg.ipHeader).toBe("X-Real-IP");
    expect(cfg.keepAlive).toBe(5);
    expect(cfg.logLevel).toBe("normal");
    expect(cfg.cliColors).toBe(true);
    expect(cfg.profile).toBe("debug");
  });

  it("static factories instantiate Config", () => {
    const debugCfg = Config.debugDefault();
    expect(debugCfg.profile).toBe("debug");

    const releaseCfg = Config.releaseDefault();
    expect(releaseCfg.profile).toBe("release");

    const defaultCfg = Config.defaultConfig();
    expect(defaultCfg).toBeDefined();

    const figmentCfg = Config.figment();
    expect(figmentCfg.port).toBe(8000);
  });

  it("Config.parameters() lists all parameter names", () => {
    const params = Config.parameters();
    expect(params).toContain("address");
    expect(params).toContain("port");
    expect(params).toContain("workers");
    expect(params).toContain("max_blocking");
    expect(params).toContain("keep_alive");
    expect(params).toContain("ident");
    expect(params).toContain("ip_header");
    expect(params).toContain("limits");
    expect(params).toContain("temp_dir");
    expect(params).toContain("log_level");
    expect(params).toContain("shutdown");
    expect(params).toContain("cli_colors");
  });

  it("Config.tryFrom() and Config.from() merge override options", () => {
    const init = {
      port: 9090,
      address: "0.0.0.0",
      workers: 8,
      maxBlocking: 256,
      ident: "CustomRocket",
      logLevel: "debug",
      cliColors: false,
    };

    const cfg1 = Config.tryFrom(init);
    expect(cfg1.port).toBe(9090);
    expect(cfg1.address).toBe("0.0.0.0");
    expect(cfg1.workers).toBe(8);
    expect(cfg1.maxBlocking).toBe(256);
    expect(cfg1.ident).toBe("CustomRocket");
    expect(cfg1.logLevel).toBe("debug");
    expect(cfg1.cliColors).toBe(false);

    const cfg2 = Config.from(init);
    expect(cfg2.port).toBe(9090);
  });

  it("Config getters and setters inspect and mutate fields", () => {
    const cfg = new Config();

    cfg.port = 8080;
    expect(cfg.port).toBe(8080);

    cfg.address = "127.0.0.2";
    expect(cfg.address).toBe("127.0.0.2");

    expect(() => {
      cfg.address = "not_an_ip";
    }).toThrow();

    cfg.workers = 16;
    expect(cfg.workers).toBe(16);

    cfg.maxBlocking = 1024;
    expect(cfg.maxBlocking).toBe(1024);

    cfg.ident = "NewIdent";
    expect(cfg.ident).toBe("NewIdent");

    cfg.ident = null;
    expect(cfg.ident).toBeNull();

    cfg.ipHeader = "X-Forwarded-For";
    expect(cfg.ipHeader).toBe("X-Forwarded-For");

    cfg.tempDir = "/tmp/rocket";
    expect(cfg.tempDir).toBe("/tmp/rocket");

    cfg.keepAlive = 10;
    expect(cfg.keepAlive).toBe(10);

    cfg.logLevel = "critical";
    expect(cfg.logLevel).toBe("critical");

    cfg.cliColors = false;
    expect(cfg.cliColors).toBe(false);

    cfg.limits = {
      limits: {
        form: "64KiB",
        json: "10MiB",
      },
    };
    expect(cfg.limits.limits).toBeDefined();

    cfg.shutdown = {
      ctrlc: false,
      grace: 3,
      mercy: 4,
      force: true,
      signals: ["term", "hup"],
    };
    expect(cfg.shutdown.grace).toBe(3);
    expect(cfg.shutdown.ctrlc).toBe(false);
  });
});

import { describe, expect, it } from "vite-plus/test";
import { Config, DEBUG_PROFILE, DEFAULT_PROFILE, RELEASE_PROFILE } from "../dist/index.js";

describe("Config", () => {
  it("exposes profile constants", () => {
    expect(DEBUG_PROFILE).toBe("debug");
    expect(RELEASE_PROFILE).toBe("release");
    expect(DEFAULT_PROFILE).toBe("debug");
  });

  it("creates debug and release default configs", () => {
    const debugCfg = Config.debugDefault();
    expect(debugCfg.address).toBe("127.0.0.1");
    expect(debugCfg.port).toBe(8000);
    expect(debugCfg.logLevel).toBe("normal");

    const releaseCfg = Config.releaseDefault();
    expect(releaseCfg.address).toBe("127.0.0.1");
    expect(releaseCfg.port).toBe(8000);
    expect(releaseCfg.logLevel).toBe("critical");
  });

  it("lists config parameters", () => {
    const params = Config.parameters();
    expect(params).toContain("address");
    expect(params).toContain("port");
    expect(params).toContain("workers");
    expect(params).toContain("limits");
    expect(params).toContain("shutdown");
  });

  it("allows setting and getting fields on Config instance", () => {
    const cfg = new Config();
    cfg.port = 8080;
    expect(cfg.port).toBe(8080);

    cfg.address = "0.0.0.0";
    expect(cfg.address).toBe("0.0.0.0");

    cfg.workers = 8;
    expect(cfg.workers).toBe(8);

    cfg.logLevel = "debug";
    expect(cfg.logLevel).toBe("debug");

    cfg.cliColors = false;
    expect(cfg.cliColors).toBe(false);

    cfg.keepAlive = 10;
    expect(cfg.keepAlive).toBe(10);
  });

  it("supports tryFrom with custom config overrides", () => {
    const cfg = Config.tryFrom({
      port: 9000,
      address: "127.0.0.1",
      workers: 16,
      logLevel: "critical",
      shutdown: {
        ctrlc: false,
        grace: 10,
      },
    });

    expect(cfg.port).toBe(9000);
    expect(cfg.address).toBe("127.0.0.1");
    expect(cfg.workers).toBe(16);
    expect(cfg.logLevel).toBe("critical");
    expect(cfg.shutdown.ctrlc).toBe(false);
    expect(cfg.shutdown.grace).toBe(10);
  });

  it("validates invalid address or log level", () => {
    const cfg = new Config();
    expect(() => {
      cfg.address = "invalid-ip";
    }).toThrow();

    expect(() => {
      cfg.logLevel = "invalid-level";
    }).toThrow();
  });
});

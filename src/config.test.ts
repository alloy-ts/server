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
  SHUTDOWN,
  TEMP_DIR,
  WORKERS,
} from "../dist/index.js";

test("Config constants are defined", () => {
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
});

test("Config static constructors and instance getters/setters", () => {
  const debugCfg = Config.debugDefault();
  expect(debugCfg.profile).toBe("debug");
  expect(debugCfg.port).toBe(8000);
  expect(debugCfg.address).toBe("127.0.0.1");

  const releaseCfg = Config.releaseDefault();
  expect(releaseCfg.profile).toBe("release");

  const figmentCfg = Config.figment();
  expect(figmentCfg).toBeDefined();

  const customCfg = Config.tryFrom({
    port: 9090,
    address: "0.0.0.0",
    logLevel: "critical",
  });
  expect(customCfg.port).toBe(9090);
  expect(customCfg.address).toBe("0.0.0.0");
  expect(customCfg.logLevel).toBe("critical");

  customCfg.port = 8080;
  expect(customCfg.port).toBe(8080);
});

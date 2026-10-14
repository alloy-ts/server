/**
 * Configuration Demo for Rocket NAPI Bindings
 *
 * Demonstrates:
 * - `Config.debugDefault()` and `Config.releaseDefault()`
 * - `Config.tryFrom()` partial overrides
 * - Reading & mutating configuration fields (`port`, `address`, `logLevel`, `shutdown`)
 * - Launching a Rocket server instance with a custom `Config`
 */

import { Config, Rocket } from "../dist/index.js";

console.log("=== Rocket Default Parameters ===");
console.log(Config.parameters().join(", "));

console.log("\n=== Debug Defaults ===");
const debugCfg = Config.debugDefault();
console.log("Profile: ", debugCfg.profile);
console.log("Port:    ", debugCfg.port);
console.log("Address: ", debugCfg.address);
console.log("LogLevel:", debugCfg.logLevel);

console.log("\n=== Release Defaults ===");
const releaseCfg = Config.releaseDefault();
console.log("Profile: ", releaseCfg.profile);
console.log("LogLevel:", releaseCfg.logLevel);

console.log("\n=== Config.tryFrom() Partial Override ===");
const customCfg = Config.tryFrom({
  port: 8080,
  address: "0.0.0.0",
  logLevel: "debug",
  shutdown: {
    grace: 3,
    mercy: 5,
  },
});

console.log("Custom Port:    ", customCfg.port);
console.log("Custom Address: ", customCfg.address);
console.log("Custom LogLevel:", customCfg.logLevel);
console.log("Custom Shutdown:", customCfg.shutdown);

// Mutate options directly
customCfg.port = 9000;
customCfg.cliColors = false;
console.log("Updated Port:    ", customCfg.port);
console.log("Updated Colors:  ", customCfg.cliColors);

// Create a Rocket server with the custom configuration
const rocket = new Rocket({
  address: customCfg.address,
  port: customCfg.port,
  logLevel: customCfg.logLevel,
});

rocket.mount("/", "GET", "/health", async () => ({
  status: 200,
  headers: [{ name: "Content-Type", value: "application/json" }],
  body: Buffer.from(JSON.stringify({ status: "healthy" })),
}));

console.log("\nRocket server configured and ready to launch at port", customCfg.port);

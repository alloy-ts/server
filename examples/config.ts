/**
 * Rocket Server Configuration (`Config`) Example
 *
 * Demonstrates constructing and customizing Rocket server configuration
 * using debug/release defaults or `Config.tryFrom()`.
 */

import { Config, Rocket } from "../src/main.ts";

console.log("=== Rocket Configuration Example ===");

// 1. Inspect supported parameter names
console.log("Supported parameters:", Config.parameters().join(", "));

// 2. Create default configuration for debug mode
const config = Config.debugDefault();
config.port = 8080;
config.address = "127.0.0.1";
config.logLevel = "normal";

console.log(`Configured address: ${config.address}:${config.port}`);
console.log(`Log level: ${config.logLevel}`);

// 3. Create configuration using Config.tryFrom
const customConfig = Config.tryFrom({
  port: 8000,
  address: "0.0.0.0",
  workers: 4,
  maxBlocking: 256,
  logLevel: "normal",
  shutdown: {
    ctrlc: true,
    grace: 5,
    mercy: 5,
  },
  limits: {
    limits: {
      json: "2MiB",
      form: "64KiB",
    },
  },
});

console.log("Custom Config created:");
console.log(`- Address: http://${customConfig.address}:${customConfig.port}`);
console.log(`- Workers: ${customConfig.workers}`);
console.log(`- Grace period: ${customConfig.shutdown.grace}s`);

// 4. Instantiate a Rocket server with the configuration
const rocket = new Rocket({
  port: customConfig.port,
  address: customConfig.address,
  logLevel: customConfig.logLevel,
});

rocket.mount("/", "GET", "/api/info", () => ({
  status: 200,
  headers: [{ name: "Content-Type", value: "application/json" }],
  body: Buffer.from(JSON.stringify({ status: "running", port: customConfig.port })),
}));

console.log("Rocket server ready to launch.");

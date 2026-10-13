/**
 * Rocket Configuration (`Config` / `Rocket`) Example
 *
 * Demonstrates Rocket server configuration and lifecycle control using
 * `@alloy-ts/server`.
 */

import { Config, Rocket, response } from "@alloy-ts/server";

// 1. Inspect default debug configuration
const debugConfig = Config.debugDefault();
console.log("Default debug configuration:");
console.log(`- Profile: ${debugConfig.profile}`);
console.log(`- Address: ${debugConfig.address}`);
console.log(`- Port: ${debugConfig.port}`);
console.log(`- Log Level: ${debugConfig.logLevel}`);
console.log(`- Identity: ${debugConfig.ident}`);

// 2. Build custom configuration with partial overrides
const customConfig = Config.tryFrom({
  port: 8080,
  address: "127.0.0.1",
  logLevel: "normal",
  ident: "CustomRocket/1.0",
  limits: {
    limits: {
      json: "10MiB",
      form: "64kB",
    },
  },
  shutdown: {
    ctrlc: true,
    grace: 2,
    mercy: 3,
  },
});

console.log("\nCustom configuration created:");
console.log(`- Port: ${customConfig.port}`);
console.log(`- Identity: ${customConfig.ident}`);

// 3. Initialize Rocket server using custom config
const rocket = new Rocket({
  port: customConfig.port,
  address: customConfig.address,
  ident: customConfig.ident ?? undefined,
  logLevel: customConfig.logLevel,
});

// Mount route handlers
rocket.mount("/", "GET", "/health", async () => {
  return {
    status: 200,
    headers: [{ name: "Content-Type", value: "application/json" }],
    body: Buffer.from(JSON.stringify({ status: "ok", timestamp: Date.now() })),
  };
});

console.log("\nLaunching Rocket server with custom configuration...");

// Launch server in background
rocket.launch().then(() => {
  console.log("Rocket server shut down gracefully.");
});

// Shutdown gracefully after 1 second for demonstration
setTimeout(() => {
  console.log("Initiating shutdown...");
  rocket.shutdown();
}, 1000);

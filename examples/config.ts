import { Config, DEBUG_PROFILE, RELEASE_PROFILE } from "../index.js";

// 1. Inspect debug defaults
const debugConfig = Config.debugDefault();
console.log("Debug Profile:", debugConfig.profile);
console.log("Debug Port:", debugConfig.port);
console.log("Debug Address:", debugConfig.address);
console.log("Debug Workers:", debugConfig.workers);
console.log("Debug Log Level:", debugConfig.logLevel);

// 2. Inspect release defaults
const releaseConfig = Config.releaseDefault();
console.log("\nRelease Profile:", releaseConfig.profile);
console.log("Release Log Level:", releaseConfig.logLevel);

// 3. Build a custom Config with overrides using `Config.from`
const customConfig = Config.from({
  profile: "staging",
  address: "127.0.0.1",
  port: 8080,
  workers: 8,
  logLevel: "normal",
  cliColors: true,
  limits: {
    limits: {
      json: "10MiB",
      form: "64KiB",
    },
  },
  shutdown: {
    ctrlc: true,
    grace: 10,
    mercy: 5,
  },
});

console.log("\nCustom Config Profile:", customConfig.profile);
console.log("Custom Config Port:", customConfig.port);
console.log("Custom Config Address:", customConfig.address);
console.log("Custom Config Limits:", customConfig.limits);

// Mutate configuration fields directly
customConfig.port = 9000;
console.log("Updated Custom Port:", customConfig.port);

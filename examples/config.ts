import { Config, ConfigInit } from "../dist/index.js";

// 1. Inspect Default Rocket Configurations
const debugConfig = Config.debugDefault();
console.log("--- Debug Default Config ---");
console.log(`Profile: ${debugConfig.profile}`);
console.log(`Address: ${debugConfig.address}`);
console.log(`Port: ${debugConfig.port}`);
console.log(`Log Level: ${debugConfig.logLevel}`);
console.log(`CLI Colors: ${debugConfig.cliColors}`);

const releaseConfig = Config.releaseDefault();
console.log("\n--- Release Default Config ---");
console.log(`Profile: ${releaseConfig.profile}`);
console.log(`Port: ${releaseConfig.port}`);

// 2. Programmatic Configuration Overrides using Config.tryFrom()
const customInit: ConfigInit = {
  port: 8080,
  address: "0.0.0.0",
  logLevel: "debug",
  keepAlive: 15,
  shutdown: {
    ctrlc: true,
    grace: 5,
    mercy: 5,
    signals: ["term", "hup", "int"],
  },
  limits: {
    limits: {
      json: "10MiB",
      form: "64KiB",
    },
  },
};

const customConfig = Config.tryFrom(customInit);
console.log("\n--- Custom Overridden Config ---");
console.log(`Address: ${customConfig.address}`);
console.log(`Port: ${customConfig.port}`);
console.log(`Log Level: ${customConfig.logLevel}`);
console.log(`Keep Alive: ${customConfig.keepAlive}s`);
console.log(`Shutdown Grace Period: ${customConfig.shutdown.grace}s`);

// 3. Mutate Configuration Instance
customConfig.port = 9000;
customConfig.address = "127.0.0.1";
console.log("\n--- Mutated Config ---");
console.log(`New Server Endpoint: http://${customConfig.address}:${customConfig.port}`);

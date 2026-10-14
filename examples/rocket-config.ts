import { Config } from "../src/main.ts";

// 1. Inspect debug defaults
const debugConfig = Config.debugDefault();
console.log("Debug Config Defaults:");
console.log(`  Address: ${debugConfig.address}`);
console.log(`  Port: ${debugConfig.port}`);
console.log(`  Workers: ${debugConfig.workers}`);
console.log(`  Log Level: ${debugConfig.logLevel}`);

// 2. Create custom configuration using tryFrom
const customConfig = Config.tryFrom({
  address: "0.0.0.0",
  port: 8080,
  workers: 8,
  logLevel: "debug",
  ident: "AlloyRocketServer",
  limits: {
    limits: {
      json: "10MiB",
      form: "64KiB",
    },
  },
  shutdown: {
    grace: 5,
    mercy: 5,
    ctrlc: true,
  },
});

console.log("\nCustom Config Overrides:");
console.log(`  Address: ${customConfig.address}`);
console.log(`  Port: ${customConfig.port}`);
console.log(`  Workers: ${customConfig.workers}`);
console.log(`  Ident: ${customConfig.ident}`);
console.log(`  Json Limit: ${customConfig.limits.limits["json"]}`);

// 3. Inspect all supported configuration parameter names
console.log("\nSupported Configuration Parameters:");
console.log(Config.parameters());

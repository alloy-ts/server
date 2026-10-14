import { Config } from "../dist/index.js";

// Programmatic Rocket configuration management
const config = Config.debugDefault();
config.port = 8080;
config.address = "0.0.0.0";
config.workers = 8;
config.logLevel = "normal";

console.log(`Configured Rocket to run on ${config.address}:${config.port}`);
console.log(`Workers: ${config.workers}, Log level: ${config.logLevel}`);

// Merging overrides
const customConfig = Config.tryFrom({
  port: 9090,
  workers: 16,
  cliColors: true,
});

console.log(`Custom Config port: ${customConfig.port}`);

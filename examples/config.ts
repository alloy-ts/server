import { Config, DEBUG_PROFILE, RELEASE_PROFILE } from "../dist/index.js";

console.log("=== Rocket Configuration Profiles ===");
console.log("Debug Profile:", DEBUG_PROFILE);
console.log("Release Profile:", RELEASE_PROFILE);

console.log("\n=== Default Debug Config ===");
const defaultConfig = new Config();
console.log("Profile:", defaultConfig.profile);
console.log("Port:", defaultConfig.port);
console.log("Address:", defaultConfig.address);
console.log("Workers:", defaultConfig.workers);
console.log("Max Blocking Threads:", defaultConfig.maxBlocking);
console.log("Keep Alive:", defaultConfig.keepAlive, "seconds");
console.log("Ident Header:", defaultConfig.ident);
console.log("Log Level:", defaultConfig.logLevel);

console.log("\n=== Customizing Configuration ===");
const customConfig = Config.tryFrom({
  port: 8080,
  address: "0.0.0.0",
  workers: 16,
  logLevel: "normal",
  ident: "MyAlloyRocketServer",
  shutdown: {
    ctrlc: true,
    signals: ["term", "int", "hup"],
    grace: 5,
    mercy: 5,
  },
  limits: {
    limits: {
      json: "10MiB",
      form: "64KiB",
    },
  },
});

console.log("Custom Port:", customConfig.port);
console.log("Custom Address:", customConfig.address);
console.log("Custom Workers:", customConfig.workers);
console.log("Custom Ident:", customConfig.ident);
console.log("Custom Log Level:", customConfig.logLevel);

console.log("\n=== Modifying Config Properties ===");
customConfig.port = 9090;
customConfig.logLevel = "debug";
console.log("Updated Port:", customConfig.port);
console.log("Updated Log Level:", customConfig.logLevel);

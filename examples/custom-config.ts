import { Config, Rocket } from "../dist/index.js";

/**
 * Demonstrates configuring Rocket via the NAPI Config class.
 */
const config = new Config();
config.port = 8085;
config.address = "127.0.0.1";
config.workers = 4;
config.logLevel = "normal";

config.limits = {
  limits: {
    form: "32KiB",
    json: "5MiB",
  },
};

config.shutdown = {
  ctrlc: true,
  grace: 3,
  mercy: 3,
};

console.log("Configured Rocket parameters:");
console.log(`- Address: ${config.address}:${config.port}`);
console.log(`- Workers: ${config.workers}`);
console.log(`- JSON limit: ${config.limits.limits["json"]}`);

const rocket = Rocket.custom({
  port: config.port,
  address: config.address,
  workers: config.workers,
});

console.log(`Configured Rocket ready on port ${rocket.config.port}`);

import { Config, Rocket } from "../dist/index.js";

// Create a custom Rocket configuration using Config.tryFrom or Config instance
const config = Config.tryFrom({
  port: 8080,
  address: "127.0.0.1",
  ident: "MyRocketServer/1.0",
  logLevel: "debug",
  cliColors: true,
  keepAlive: 15,
  workers: 8,
  limits: {
    limits: {
      json: "2MiB",
      form: "64KiB",
    },
  },
  shutdown: {
    ctrlc: true,
    grace: 5,
    mercy: 5,
    force: false,
  },
});

console.log("Configured Profile:", config.profile);
console.log("Port:", config.port);
console.log("Address:", config.address);
console.log("Ident:", config.ident);
console.log("Limits:", config.limits.limits);

// Create a Rocket application with this configuration
const rocket = new Rocket({
  port: config.port,
  address: config.address,
  ident: config.ident ?? undefined,
  logLevel: config.logLevel,
});

console.log("Rocket app initialized with custom configuration.");

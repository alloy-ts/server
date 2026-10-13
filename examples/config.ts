import { Config, serve, Response } from "../src/main.ts";

console.log("Supported Config parameters:");
console.log(Config.parameters());

const debugConfig = Config.debugDefault();
console.log(`Debug profile default port: ${debugConfig.port}`);

const releaseConfig = Config.releaseDefault();
console.log(`Release profile default port: ${releaseConfig.port}`);

const config = Config.tryFrom({
  port: 8080,
  address: "127.0.0.1",
  workers: 8,
  logLevel: "normal",
  shutdown: {
    grace: 5,
    mercy: 5,
    ctrlc: true,
  },
});

console.log(`Configured server on ${config.address}:${config.port} with ${config.workers} workers`);

const server = serve({
  port: config.port,
  hostname: config.address,
  fetch() {
    return new Response(`Serving from Rocket on port ${config.port}`);
  },
});

console.log(`Server started at ${server.url}`);

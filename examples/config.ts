import { Config, Limits, Shutdown, serve, Response } from "../dist/index.js";

// 1. Create a custom configuration using Figment defaults
const config = Config.debugDefault();
config.port = 8080;
config.address = "127.0.0.1";
config.ident = "Rocket-TS/1.0";
config.keepAlive = 15;

// 2. Custom streaming limits (e.g. 10MB JSON payload limit)
config.limits = new Limits({
  json: 10 * 1024 * 1024,
  form: 64 * 1024,
});

// 3. Graceful shutdown settings
config.shutdown = new Shutdown({
  ctrlc: true,
  grace: 10,
  mercy: 5,
  force: true,
});

console.log("Configured Rocket Server Parameters:");
console.log(`Port: ${config.port}`);
console.log(`Address: ${config.address}`);
console.log(`Server Ident: ${config.ident}`);
console.log(`JSON Limit: ${config.limits.json} bytes`);

// 4. Start server using configured options
const server = serve({
  port: config.port,
  hostname: config.address,
  routes: {
    "/config": () =>
      Response.json({
        port: config.port,
        address: config.address,
        ident: config.ident,
      }),
  },
});

console.log(`Server started at ${server.url}`);

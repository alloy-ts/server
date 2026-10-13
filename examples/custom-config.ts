/**
 * Example: Custom Rocket Configuration & Server Launch
 *
 * Demonstrates:
 * 1. Setting up Rocket configuration using `Config`
 * 2. Customizing address, port, worker count, and size limits
 * 3. Launching `Alloy.serve` with custom configuration options
 */

import { Alloy, Config, Response } from "../dist/main.mjs";

// 1. Create and customize Rocket configuration
const config = Config.debugDefault();
config.port = 8080;
config.address = "127.0.0.1";
config.workers = 8;
config.limits = {
  limits: {
    json: "5MiB",
    form: "64KiB",
  },
};
config.logLevel = "normal";

console.log(`Configured Port: ${config.port}`);
console.log(`Configured Address: ${config.address}`);
console.log(`Configured Workers: ${config.workers}`);

// 2. Launch Alloy.serve with customized options
const server = Alloy.serve({
  port: config.port,
  hostname: config.address,
  routes: {
    "/": new Response("Custom Configured Rocket Server"),
    "/api/info": Response.json({
      port: config.port,
      address: config.address,
      workers: config.workers,
    }),
  },
});

console.log(`Server running at ${server.url}`);

// Graceful shutdown after 3 seconds for demo purposes
setTimeout(() => {
  console.log("Stopping server...");
  server.stop();
}, 3000);

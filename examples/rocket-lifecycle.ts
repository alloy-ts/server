/**
 * Programmatic Rocket Server Lifecycle Example
 *
 * Demonstrates direct programmatic control over mounting routes, registering error
 * catchers, ignition, launch, and graceful shutdown.
 */

import { Rocket } from "../dist/index.js";

console.log("=== Rocket Lifecycle Example ===");

// 1. Create a new Rocket instance
const rocket = new Rocket({
  port: 8080,
  logLevel: "normal",
});

// 2. Mount custom route handlers
rocket.mount("/", "GET", "/api/greet/:name", (req) => {
  return {
    status: 200,
    headers: [{ name: "Content-Type", value: "text/plain" }],
    body: Buffer.from(`Hello, ${req.params?.name}!`),
  };
});

// 3. Register custom error catcher
rocket.register("/", 404, () => {
  return {
    status: 404,
    headers: [{ name: "Content-Type", value: "application/json" }],
    body: Buffer.from(JSON.stringify({ error: "Custom 404 Not Found" })),
  };
});

console.log("Mounted routes and catchers. Launching Rocket server...");

// 4. Launch the server in a non-blocking background task
const launchPromise = rocket.launch();

// 5. Gracefully shutdown after a short delay
setTimeout(() => {
  console.log("Initiating graceful shutdown...");
  rocket.shutdown();
}, 1000);

await launchPromise;
console.log("Rocket server shutdown completed.");
